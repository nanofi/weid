#![feature(plugin)]
#![feature(try_blocks)]
#![feature(slice_index_methods)]

extern crate futures;
#[macro_use] extern crate actix;
extern crate actix_web;
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
use actix_web::{self as web, server, http};

use self::config::Config;
use self::db::{Article, Db};

struct AppState {
  db: Addr<Db>,
}

type Request = web::HttpRequest<Arc<AppState>>;
type Response = web::HttpResponse;
type WebResult<T> = web::error::Result<T>;

fn index(_ : web::Path<()>) -> WebResult<web::fs::NamedFile> {
  Ok(web::fs::NamedFile::open(Path::new("index.html"))?)
}

fn favicon(_ : web::Path<()>) -> WebResult<web::fs::NamedFile> {
  Ok(web::fs::NamedFile::open(Path::new("favicon.ico"))?)
}

fn search(req: &Request) -> WebResult<web::Json<Vec<Article>>> {
  let state = req.state();
  let query = req.query();
  let search = query.get("q").map(|v| (*v).as_ref()).unwrap_or("");

  let results = state.db.send(db::Search::new(search)).wait()??;
  
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

impl web::error::ResponseError for DataUrlError {
  fn error_response(&self) -> Response {
    Response::BadRequest()
      .finish()
  }
}

fn add(state: web::State<Arc<AppState>>, values: web::Json<AddValues>) -> WebResult<web::Json<Article>> {
  let (file, _) =  DataUrl::process(values.file.as_str())
    .map_err(|_| DataUrlError::InvalidFormat)?
    .decode_to_vec()
    .map_err(|_| DataUrlError::InvalidContent)?;
  
  let result = state.db.send(db::Add::new(values.title.clone(), values.authors.clone(), file)).wait()??;
  
  Ok(web::Json(result))
}

fn delete(state: web::State<Arc<AppState>>, path: web::Path<(Uuid)>) -> WebResult<web::Json<Article>> {
  let result = state.db.send(db::Remove::new(*path)).wait()??;
  Ok(web::Json(result))
}

fn view(state: web::State<Arc<AppState>>, path: web::Path<(Uuid)>) -> WebResult<web::fs::NamedFile> {
  let result = state.db.send(db::Get::new(*path)).wait()??;
  let file = web::fs::NamedFile::open(result.path())?
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
  
  let state = Arc::new(AppState {
    db: Db::open(config.db_path)?
  });
  
  let upload_limit = config.upload_limit;
  server::new(move || {
    web::App::with_state(state.clone())
      .middleware(web::middleware::Logger::default())
      .handler("/assets", web::fs::StaticFiles::new("assets").unwrap().show_files_listing())
      .resource("/", |r| r.get().with(index))
      .resource("/favicon.ico", |r| r.get().with(favicon))
      .resource("/search", |r| r.get().f(search))
      .resource("/add", move |r| r.post().with_config(add, move |cfg| {
        cfg.1.limit(upload_limit);
      }))
      .resource("/delete/{id}", |r| r.post().with(delete))
      .resource("/view/{id}", |r| r.get().with(view))
  })
    .workers(config.workers)
    .bind((config.address.as_str(), config.port))?
    .start();
  
  let _ = sys.run();
  
  Ok(())
}
