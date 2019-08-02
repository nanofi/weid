#![feature(plugin)]
#![feature(try_blocks)]
#![feature(slice_index_methods)]
#![feature(rustc_private)]
#![feature(box_syntax)]
#![feature(let_chains)]
#![feature(generators, generator_trait)]

extern crate actix;
extern crate actix_files;
extern crate actix_multipart;
extern crate actix_web;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate toml;
#[macro_use]
extern crate log;
extern crate num_cpus;
extern crate simplelog;
#[macro_use]
extern crate failure;
extern crate lmdb_zero as lmdb;
extern crate tempfile;
#[cfg(test)]
extern crate rand;

mod collection;
mod config;
mod db;

use std::fs::{remove_file, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;

use actix::Addr;
use actix_files as fs;
use actix_multipart::{Field, Multipart, MultipartError};
use actix_web::{error, http, middleware, web, App, HttpServer, Responder};
use failure::Error;
use futures::{future, Future, Stream};
use tempfile::NamedTempFile;

use self::config::Config;
use self::db::Db;

struct AppData {
  db: Addr<Db>,
}

fn index() -> impl Responder {
  fs::NamedFile::open(Path::new("index.html"))
}

fn favicon() -> impl Responder {
  fs::NamedFile::open(Path::new("favicon.ico"))
}

struct AddParam {
  title: Option<String>,
  authors: Option<Vec<String>>,
  file: Option<NamedTempFile>,
}
impl AddParam {
  fn new() -> Self {
    Self {
      title: None,
      authors: None,
      file: None,
    }
  }
}

fn read_json_field<T>(field: Field) -> impl Future<Item = T, Error = error::Error>
where
  for<'a> T: serde::Deserialize<'a>,
{
  field
    .map_err(|e| error::ErrorInternalServerError(e))
    .fold(web::BytesMut::with_capacity(8192), |mut body, chunk| {
      if (body.len() + chunk.len()) > 32_768 {
        Err(format_err!("Overflow field"))
      } else {
        body.extend_from_slice(&chunk);
        Ok(body)
      }
    })
    .and_then(|body| serde_json::from_slice(&body).map_err(|e| error::ErrorInternalServerError(e)))
}

fn read_file_field(field: Field) -> impl Future<Item = NamedTempFile, Error = error::Error> {
  let file = match NamedTempFile::new() {
    Ok(file) => file,
    Err(e) => return future::Either::A(future::err(error::ErrorInternalServerError(e))),
  };
  future::Either::B(
    field
      .fold(file, |mut file, chunk| {
        web::block(move || {
          file
            .write_all(chunk.as_ref())
            .map_err(|e| MultipartError::Payload(error::PayloadError::Io(e)))?;
          Ok(file)
        })
        .map_err(|e: error::BlockingError<MultipartError>| match e {
          error::BlockingError::Error(e) => e,
          error::BlockingError::Canceled => MultipartError::Incomplete,
        })
      })
      .map_err(|e| error::ErrorInternalServerError(e)),
  )
}

fn add(
  data: web::Data<Arc<AppData>>,
  multipart: Multipart,
) -> impl Future<Item = impl Responder, Error = error::Error> {
  multipart
    .map_err(|e| error::ErrorInternalServerError(e))
    .fold(
      AddParam::new(),
      |mut param, field| -> Box<dyn Future<Item = _, Error = _>> {
        let cd = match field.content_disposition() {
          Some(cd) => cd,
          None => {
            return box future::err(error::ErrorInternalServerError(format_err!(
              "The content disposition header is required."
            )))
          }
        };
        let name = match cd.get_name() {
          Some(name) => name,
          None => {
            return box future::err(error::ErrorInternalServerError(format_err!(
              "The name is required."
            )))
          }
        };
        match name {
          "title" => box read_json_field(field).and_then(|val| {
            param.title = Some(val);
            box future::ok(param)
          }),
          "authors" => box read_json_field(field).and_then(|val| {
            param.authors = Some(val);
            future::ok(param)
          }),
          "file" => box read_file_field(field).and_then(|file| {
            param.file = Some(file);
            future::ok(param)
          }),
          _ => box future::err(error::ErrorInternalServerError(format_err!("Unknown name"))),
        }
      },
    )
    .and_then(|param| {
      let title = match param.title {
        Some(val) => val,
        None => {
          return Err(error::ErrorInternalServerError(format_err!(
            "'title' is not provided"
          )))
        }
      };
      let authors = match param.authors {
        Some(val) => val,
        None => {
          return Err(error::ErrorInternalServerError(format_err!(
            "'authors' is not provided"
          )))
        }
      };
      let file = match param.file {
        Some(val) => val,
        None => {
          return Err(error::ErrorInternalServerError(format_err!(
            "'file' is not provided"
          )))
        }
      };
      Ok((title, authors, file))
    })
    .and_then(move |(title, authors, file)| {
      data.db.send(db::Add::new(title, authors)).then(|result| {
        let article = result??;
        file
          .persist(article.path())
          .map_err(|_| format_err!("The file failed to move"))?;
        Ok(web::Json(article))
      })
    })
}

fn delete(
  data: web::Data<Arc<AppData>>,
  path: web::Path<(u64)>,
) -> impl Future<Item = impl Responder, Error = error::Error> {
  data.db.send(db::Remove::new(*path)).then(|result| {
    let article = result??;
    remove_file(article.path())?;
    Ok(web::Json(article))
  })
}

fn view(
  data: web::Data<Arc<AppData>>,
  path: web::Path<(u64)>,
) -> impl Future<Item = impl Responder, Error = error::Error> {
  data.db.send(db::Get::new(*path)).then(|result| {
    let article = result??;
    let file = fs::NamedFile::open(article.path())?.set_content_disposition(
      http::header::ContentDisposition {
        disposition: http::header::DispositionType::Inline,
        parameters: vec![http::header::DispositionParam::Filename(article.filename())],
      },
    );
    Ok(file)
  })
}

#[derive(Deserialize, Debug)]
struct SearchQuery {
  q: String,
}

fn search(
  data: web::Data<Arc<AppData>>,
  query: web::Query<SearchQuery>,
) -> impl Future<Item = impl Responder, Error = error::Error> {
  let search = db::Search::new(&query.q);
  data.db.send(search).then(|result| Ok(web::Json(result??)))
}

fn load_config() -> Result<Config, Error> {
  {
    let path = Path::new("Config.toml");
    if path.exists() {
      let mut file = OpenOptions::new().read(true).open(path)?;
      let mut buffer = Vec::new();
      file.read_to_end(&mut buffer)?;
      let config = toml::from_slice(buffer.as_slice())?;
      return Ok(config);
    }
  }
  Ok(Default::default())
}

fn main() -> Result<(), Error> {
  let config = load_config()?;
  simplelog::TermLogger::init(
    config.log,
    simplelog::Config::default(),
    simplelog::TerminalMode::Mixed,
  )?;

  let system = actix::System::new("weid");

  let data = Arc::new(AppData {
    db: Db::open(config.db_path)?,
  });

  HttpServer::new(move || {
    App::new()
      .data(data.clone())
      .wrap(middleware::Logger::default())
      .service(fs::Files::new("/assets", "assets"))
      .route("/", web::get().to(index))
      .route("/favicon.ico", web::get().to(favicon))
      .route("/search", web::get().to_async(search))
      .route("/add", web::post().to_async(add))
      .route("/delete/{id}", web::post().to_async(delete))
      .route("/view/{id}", web::get().to_async(view))
  })
  .workers(config.workers)
  .bind((config.address.as_str(), config.port))?
  .start();

  Ok(system.run()?)
}
