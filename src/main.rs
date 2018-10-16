#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(custom_derive)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate url;
extern crate url_serde;

use std::path::Path;
use std::path::PathBuf;

use rocket::fairing::AdHoc;
use rocket::State;
use rocket::response::NamedFile;

use rocket_contrib::Json;

use url::Url;

struct StaticDir(String);

#[derive(FromForm)]
struct Query {
    q: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AddValues {
    title: String,
    authors: Vec<String>,
    #[serde(with = "url_serde")]
    file: Url,
}

#[derive(Serialize, Deserialize, Debug)]
struct Article {
    id: u64,
    title: String,
    authors: Vec<String>,
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
fn search(query: Query) -> Json<Vec<Article>> {
    let results = vec![
        Article { id: 1, title: "Awesome title 1".to_string(), authors: vec!["First author 1".to_string(), "second author 1".to_string()]},
        Article { id: 2, title: "Awesome title 2".to_string(), authors: vec!["First author 2".to_string(), "second author 2".to_string()]},
        Article { id: 3, title: "Awesome title 3".to_string(), authors: vec!["First author 3".to_string(), "second author 3".to_string()]},
        Article { id: 4, title: "Awesome title 4".to_string(), authors: vec!["First author 4".to_string(), "second author 4".to_string()]},
        Article { id: 5, title: "Awesome title 5".to_string(), authors: vec!["First author 5".to_string(), "second author 5".to_string()]},
        Article { id: 6, title: "Awesome title 6".to_string(), authors: vec!["First author 6".to_string(), "second author 6".to_string()]},
        Article { id: 7, title: query.q, authors: vec!["First author".to_string()]},
    ];
    Json(results)
}

#[post("/add", format = "application/json", data = "<values>")]
fn add(values: Json<AddValues>) -> Json<Article> {
    Json(Article { id: 1, title: "Test title".to_string(), authors: vec![]})
}

fn main() {
    rocket::ignite()
      .mount("/", routes![index, assets, search, add])
      .attach(AdHoc::on_attach(|rocket| {
         let static_dir = rocket.config()
           .get_str("static_dir")
           .unwrap_or("static/")
           .to_string();
         Ok(rocket.manage(StaticDir(static_dir)))
      }))
      .launch();
}
