#![feature(plugin)]
#![feature(custom_derive)]
#![feature(try_blocks)]

extern crate actix;
extern crate actix_web;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate url;
extern crate url_serde;
extern crate uuid;
#[macro_use] extern crate error_chain;
extern crate toml;
#[macro_use] extern crate log;
extern crate simplelog;
extern crate num_cpus;

mod db;
mod mmap;

use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{Read};
use std::net::ToSocketAddrs;
use std::sync::Arc;

use url::Url;
use serde::{Deserialize, Deserializer};

use actix_web::{server, http, error::Result as ActixResult, App, HttpRequest, Json, Query, fs::{NamedFile, StaticFiles}, middleware::{Logger}};

use self::db::{Article, DB};

mod errors {
    error_chain! {
        foreign_links {
            Io(std::io::Error);
            Toml(toml::de::Error);
            Log(log::SetLoggerError);
        }
    }
}
use self::errors::*;

#[derive(Deserialize, Debug)]
struct Config {
    #[serde(default = "Config::default_log")]
    #[serde(deserialize_with = "Config::deserialize_log_level")]
    log: simplelog::LevelFilter,
    #[serde(default = "Config::default_port")]
    port: u16,
    #[serde(default = "Config::default_address")]
    address: String,
    #[serde(default = "Config::default_workers")]
    workers: usize,
    #[serde(default = "Config::default_db_path")]
    db_path: PathBuf,
}
impl Config {
    fn default_log() -> simplelog::LevelFilter { simplelog::LevelFilter::Off }
    fn default_port() -> u16 { 80 }
    fn default_address() -> String { "localhost".to_owned() }
    fn default_workers() -> usize { num_cpus::get() }
    fn default_db_path() -> PathBuf { PathBuf::from("db/") }

    fn deserialize_log_level<'de, D>(deserializer: D) -> std::result::Result<simplelog::LevelFilter, D::Error>
    where D: Deserializer<'de> {
        let val = String::deserialize(deserializer)?.to_lowercase();
        let level = match val.as_str() {
            "error" => simplelog::LevelFilter::Error,
            "warn" => simplelog::LevelFilter::Warn,
            "info" => simplelog::LevelFilter::Info,
            "debug" => simplelog::LevelFilter::Debug,
            "trace" => simplelog::LevelFilter::Trace,
            _ => simplelog::LevelFilter::Off,
        };
        Ok(level)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log: simplelog::LevelFilter::Off,
            port: Self::default_port(),
            address: Self::default_address(),
            workers: Self::default_workers(),
            db_path: Self::default_db_path(),
        }
    }
}

struct AppState {
    config: Config,
}

#[derive(Deserialize)]
struct Search {
    q: String,
}

type Request = HttpRequest<Arc<AppState>>;

fn index(_ : &Request) -> ActixResult<NamedFile> {
    Ok(NamedFile::open(Path::new("index.html"))?)
}

fn favicon(_ : &Request) -> ActixResult<NamedFile> {
    Ok(NamedFile::open(Path::new("favicon.ico"))?)
}

fn search(_: Query<Search>) -> Json<Vec<Article>> {
    let results = vec![];
    Json(results)
}

/*

#[derive(Serialize, Deserialize, Debug)]
struct AddValues {
    title: String,
    authors: Vec<String>,
    #[serde(with = "url_serde")]
    file: Url,
}

struct InlineFile(NamedFile);

impl<'r> Responder<'r> for InlineFile {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        let disposition = match self.0.path().file_name() {
            Some(name) => format!("inline; filename={0}", name.to_os_string().into_string().unwrap()),
            _ => "inline".to_string()
        };
        Response::build()
            .raw_header("Content-Disposition", disposition)
            .streamed_body(self.0)
            .ok()
    }
}


#[post("/add", format = "application/json", data = "<values>")]
fn add(values: Json<AddValues>) -> Json<Article> {
    Json(Article::nil())
}

#[delete("/delete/<id>")]
fn delete(id: UUID) -> Json<Article> {
    Json(Article::nil())
}

#[get("/view/<id>")]
fn view(id: UUID) -> Result<InlineFile, NotFound<String>> {
    let file = NamedFile::open("none").map_err(|_| NotFound(format!("Bad id: {}", id)))?;
    Ok(InlineFile(file))
}
*/
    
/*
fn manage_database(rocket: Rocket) -> Result<Rocket, Rocket> {
    let path = Path::new(rocket.config().get_str("db_path").unwrap_or("db/"));
    if let Ok(db) = DB::open(path) {
        Ok(rocket.manage(Arc::new(db)))
    } else {
        Err(rocket)
    }
}
*/

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

fn init_logger(config: &Config) -> Result<()> {
    simplelog::SimpleLogger::init(config.log, simplelog::Config::default())?;
    Ok(())
}

fn main() -> Result<()> {
    let config = load_config()?;
    init_logger(&config)?;

    let address = (config.address.as_str(), config.port).to_socket_addrs()?;
    let workers = config.workers;

    let state = Arc::new(AppState {
        config: config
    });
    
    let sys = actix::System::new("weid");

    server::new(move || App::with_state(state.clone())
                .middleware(Logger::default())
                .handler("/assets", StaticFiles::new("assets").unwrap().show_files_listing())
                .resource("/", |r| r.get().f(index))
                .resource("/favicon.ico", |r| r.get().f(favicon))
                .resource("/search", |r| r.get().with(search))
    )
        .workers(workers)
        .bind(address.as_slice())?
        .start();
    
    let _ = sys.run();

    Ok(())
}
