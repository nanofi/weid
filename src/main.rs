#![feature(plugin)]
#![feature(try_blocks)]
#![feature(slice_index_methods)]

#[macro_use] extern crate actix;
extern crate actix_web;
extern crate actix_files;
extern crate actix_multipart;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate uuid;
extern crate toml;
#[macro_use] extern crate log;
extern crate simplelog;
extern crate num_cpus;
#[macro_use] extern crate failure;
extern crate lmdb_zero as lmdb;

mod db;
mod config;

use std::path::{Path};
use std::fs::OpenOptions;
use std::io::{Read};
use std::sync::Arc;

use uuid::Uuid;
use failure::Error;

use futures::{Future, Stream};
use actix::{Addr};
use actix_web::{web, middleware, http, error, App, HttpServer, Responder};
use actix_files as fs;
use actix_multipart::{Field, Multipart, MultipartError};

use self::config::Config;
use self::db::{Article, Db};

struct AppData {
  db: Addr<Db>,
}

fn index() -> impl Responder {
  fs::NamedFile::open(Path::new("index.html"))
}

fn favicon() -> impl Responder {
  fs::NamedFile::open(Path::new("favicon.ico"))
}

#[derive(Deserialize, Debug)]
struct SearchQuery {
  q: String
}

fn search(data: web::Data<Arc<AppData>>, query: web::Query<SearchQuery>) -> impl Future<Item=impl Responder, Error=error::Error> {
  let search = db::Search::new(&query.q);
  data.db.send(search).then(|result| Ok(web::Json(result??)))
}

fn add(data: web::Data<Arc<AppData>>, multipart: Multipart) -> impl Future<Item=impl Responder, Error=error::Error> {
  let title = "A";
  let authors = vec!["A".to_string(), "B".to_string()];
  let add = db::Add::new(title, authors.as_slice());
  data.db.send(add).then(|result| {
    Ok(web::Json(result??))
  })
}

fn delete(data: web::Data<Arc<AppData>>, path: web::Path<(Uuid)>) -> impl Future<Item=impl Responder, Error=error::Error> {
  data.db.send(db::Remove::new(*path)).then(|result| Ok(web::Json(result??)))
}

fn view(data: web::Data<Arc<AppData>>, path: web::Path<(Uuid)>) -> impl Future<Item=impl Responder, Error=error::Error> {
  data.db.send(db::Get::new(*path)).then(|result| {
    let article = result??;
    let file = fs::NamedFile::open(article.path())?
      .set_content_disposition(http::header::ContentDisposition {
        disposition: http::header::DispositionType::Inline,
        parameters: vec![
          http::header::DispositionParam::Filename(article.filename()),
        ],
      });
    Ok(file)
  })
}

fn load_config() -> Result<Config, Error> {
  {
    let path = Path::new("Config.toml");
    if path.exists() {
      let mut file = OpenOptions::new()
        .read(true)
        .open(path)?;
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
  simplelog::TermLogger::init(config.log, simplelog::Config::default(), simplelog::TerminalMode::Mixed)?;
  
  let sys = actix::System::new("weid");
  
  let data = Arc::new(AppData {
    db: Db::open(config.db_path)?
  });
  
  let upload_limit = config.upload_limit;
  HttpServer::new(move || {
    App::new().data(data.clone())
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
  
  let _ = sys.run();
  
  Ok(())
}
