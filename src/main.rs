#![feature(plugin)]
#![feature(try_blocks)]
#![feature(slice_index_methods)]

extern crate futures;
#[macro_use] extern crate actix;
extern crate actix_web;
extern crate actix_files;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate data_url;
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
use data_url::DataUrl;
use failure::Error;

use futures::Future;
use actix::{Addr};
use actix_web::{web, middleware, http, error, App, HttpServer, FromRequest};
use actix_files as fs;

use self::config::Config;
use self::db::{Article, Db};

struct AppData {
  db: Addr<Db>,
}

type Request = web::HttpRequest;
type Response = web::HttpResponse;
type WebResult<T> = error::Result<T>;

fn index(_ : web::Path<()>) -> WebResult<fs::NamedFile> {
  Ok(fs::NamedFile::open(Path::new("index.html"))?)
}

fn favicon(_ : web::Path<()>) -> WebResult<fs::NamedFile> {
  Ok(fs::NamedFile::open(Path::new("favicon.ico"))?)
}

#[derive(Deserialize, Debug)]
struct SearchQuery {
  q: String
}

fn search(data: web::Data<Arc<AppData>>, query: web::Query<SearchQuery>) -> WebResult<web::Json<Vec<Article>>> {
  let results = data.db.send(db::Search::new(query.q.clone())).wait()??;
  
  Ok(web::Json(results))
}


#[derive(Deserialize, Debug)]
struct AddValues {
  title: String,
  authors: Vec<String>,
  file: String,
}

#[derive(Debug, Fail)]
enum DataUrlError {
  #[fail(display = "invalid format")]
  InvalidFormat,
  #[fail(display = "invalid content")]
  InvalidContent,
}

impl error::ResponseError for DataUrlError {
  fn error_response(&self) -> Response {
    Response::BadRequest()
      .finish()
  }
}

fn add(data: web::Data<Arc<AppData>>, values: web::Json<AddValues>) -> WebResult<web::Json<Article>> {
  let (file, _) =  DataUrl::process(values.file.as_str())
    .map_err(|_| DataUrlError::InvalidFormat)?
    .decode_to_vec()
    .map_err(|_| DataUrlError::InvalidContent)?;

  let add = db::Add::new(values.title.as_str(), values.authors.as_slice(), file.as_slice());
  let request = data.db.send(add);
  let result = request.wait()??;
  
  Ok(web::Json(result))
}

fn delete(data: web::Data<Arc<AppData>>, path: web::Path<(Uuid)>) -> WebResult<web::Json<Article>> {
  let result = data.db.send(db::Remove::new(*path)).wait()??;
  Ok(web::Json(result))
}

fn view(data: web::Data<Arc<AppData>>, path: web::Path<(Uuid)>) -> WebResult<fs::NamedFile> {
  let result = data.db.send(db::Get::new(*path)).wait()??;
  let file = fs::NamedFile::open(result.path())?
    .set_content_disposition(http::header::ContentDisposition {
      disposition: http::header::DispositionType::Inline,
      parameters: vec![
        http::header::DispositionParam::Filename(result.filename()),
      ],
    });
  Ok(file)
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
  simplelog::SimpleLogger::init(config.log, simplelog::Config::default())?;
  
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
      .route("/search", web::get().to(search))
      .service(web::resource("/add")
               .data(web::Json::<AddValues>::configure(|cfg| {
                 cfg.limit(upload_limit)
               }))
               .route(web::post().to(add)))
      .route("/delete/{id}", web::post().to(delete))
      .route("/view/{id}", web::get().to(view))
  })
    .workers(config.workers)
    .bind((config.address.as_str(), config.port))?
    .start();
  
  let _ = sys.run();
  
  Ok(())
}
