#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(custom_derive)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use std::path::Path;
use std::path::PathBuf;

use rocket::fairing::AdHoc;
use rocket::State;
use rocket::response::NamedFile;

use rocket_contrib::{Json, Value};

struct StaticDir(String);

#[derive(FromForm)]
struct Query {
    q: String,
}

#[get("/")]
fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("index.html")).ok()
}

#[get("/favicon.ico")]
fn favicon() -> Option<NamedFile> {
    NamedFile::open(Path::new("favicon.ico")).ok()
}

#[get("/assets/<path..>")]
fn assets(path: PathBuf, static_dir: State<StaticDir>) -> Option<NamedFile> {
    NamedFile::open(Path::new(&static_dir.0).join(path)).ok()
}

#[get("/search?<query>")]
fn search(query: Query) -> Json<Value> {
    let results = vec![];
    Json(Value::Array(results))
}

fn main() {
    rocket::ignite()
      .mount("/", routes![index, assets, search])
      .attach(AdHoc::on_attach(|rocket| {
         let static_dir = rocket.config()
           .get_str("static_dir")
           .unwrap_or("static/")
           .to_string();
         Ok(rocket.manage(StaticDir(static_dir)))
      }))
      .launch();
}
