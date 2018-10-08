#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::path::Path;
use std::path::PathBuf;

use rocket::fairing::AdHoc;
use rocket::State;
use rocket::response::NamedFile;

struct StaticDir(String);

#[get("/")]
fn index(static_dir: State<StaticDir>) -> Option<NamedFile> {
    NamedFile::open(Path::new("index.html")).ok()
}

#[get("/assets/<path..>")]
fn assets(path: PathBuf, static_dir: State<StaticDir>) -> Option<NamedFile> {
    NamedFile::open(Path::new(&static_dir.0).join(path)).ok()
}

fn main() {
    rocket::ignite()
      .mount("/", routes![index, assets])
      .attach(AdHoc::on_attach(|rocket| {
         let static_dir = rocket.config()
           .get_str("static_dir")
           .unwrap_or("static/")
           .to_string();
         Ok(rocket.manage(StaticDir(static_dir)))
      }))
      .launch();
}
