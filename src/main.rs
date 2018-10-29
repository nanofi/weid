#![feature(plugin)]
#![feature(custom_derive)]
#![feature(try_blocks)]
#![feature(slice_index_methods)]

#[macro_use] extern crate actix;
extern crate actix_web;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate data_url;
extern crate uuid;
#[macro_use] extern crate error_chain;
extern crate toml;
#[macro_use] extern crate log;
extern crate simplelog;
extern crate num_cpus;

mod db;
mod config;

use std::path::{Path, PathBuf};
use std::fs::OpenOptions;
use std::io::{Read};
use std::sync::Arc;

use uuid::Uuid;
use data_url::DataUrl;

use actix::{Addr};
use actix_web::{self as web, server, http};

use self::config::Config;
use self::db::{Article, Db, DbOperators};

mod errors {
    error_chain! {
        links {
            Db(crate::db::Error, crate::db::ErrorKind);
        }
        foreign_links {
            Io(std::io::Error);
            Toml(toml::de::Error);
            Log(log::SetLoggerError);
        }
    }
}
use self::errors::*;

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
    let query = req.query().get("q").map(|v| (*v).as_ref()).unwrap_or("");

    let results = state.db.search(query)?;

    Ok(web::Json(results))
}


#[derive(Deserialize, Debug)]
struct AddValues {
    title: String,
    authors: Vec<String>,
    file: String,
}

fn add(state: web::State<AppState>, values: web::Json<AddValues>) -> WebResult<web::Json<Article>> {
    let mut params = db::AddParams {
        title: values.title,
        authors: values.authors,
        file: DataUrl::process(values.file.to_str())?.decode_to_vec()?,
    };
    let result = state.db.add(params)?;

    Ok(web::Json(result))
}

fn delete(state: web::State<AppState>, path: web::Path<(Uuid)>) -> WebResult<web::Json<Article>> {
    Ok(web::Json(state.db.remove(path.0)?))
}

fn view(state: web::State<AppState>, path: web::Path<(Uuid)>) -> WebResult<web::fs::NamedFile> {
    let result = state.db.get(path.0)?;
    let file = web::fs::NamedFile::open(result.path())?
        .set_content_disposition(http::header::ContentDisposition {
            disposition: http::header::DispositionType::Inline,
            parameters: vec![
                http::header::DispositionParam::Filename(result.filename()),
            ],
        });
    Ok(file)
}

fn load_config() -> Result<Config> {
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

fn main() -> Result<()> {
    let config = load_config()?;
    simplelog::SimpleLogger::init(config.log, simplelog::Config::default())?;

    let sys = actix::System::new("weid");

    let state = Arc::new(AppState {
        db: Db::open(config.db_path)?
    });

    server::new(move || web::App::with_state(state.clone())
                .middleware(web::middleware::Logger::default())
                .handler("/assets", web::fs::StaticFiles::new("assets").unwrap().show_files_listing())
                .resource("/", |r| r.get().with(index))
                .resource("/favicon.ico", |r| r.get().with(favicon))
                .resource("/search", |r| r.get().f(search))
                .resource("/add", |r| r.post().with(add))
                .resource("/delete/{id}", |r| r.post().with(delete))
                .resource("/view/{id}", |r| r.get().with(view))
    )
        .workers(config.workers)
        .bind((config.address.as_str(), config.port))?
        .start();

    let _ = sys.run();

    Ok(())
}
