#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(custom_derive)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate url;
extern crate url_serde;
extern crate uuid;

use std::path::Path;
use std::path::PathBuf;

use rocket::fairing::AdHoc;
use rocket::State;
use rocket::request::Request;
use rocket::response::{self, Response, Responder, NamedFile};
use rocket::response::status::NotFound;

use rocket_contrib::Json;
use rocket_contrib::UUID;

use url::Url;
use uuid::Uuid;

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
    id: Uuid,
    title: String,
    authors: Vec<String>,
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
        Article { id: Uuid::new_v4(), title: "Awesome title 1".to_string(), authors: vec!["First author 1".to_string(), "second author 1".to_string()]},
        Article { id: Uuid::new_v4(), title: "Awesome title 2".to_string(), authors: vec!["First author 2".to_string(), "second author 2".to_string()]},
        Article { id: Uuid::new_v4(), title: "Awesome title 3".to_string(), authors: vec!["First author 3".to_string(), "second author 3".to_string()]},
        Article { id: Uuid::new_v4(), title: "Awesome title 4".to_string(), authors: vec!["First author 4".to_string(), "second author 4".to_string()]},
        Article { id: Uuid::new_v4(), title: "Awesome title 5".to_string(), authors: vec!["First author 5".to_string(), "second author 5".to_string()]},
        Article { id: Uuid::new_v4(), title: "Awesome title 6".to_string(), authors: vec!["First author 6".to_string(), "second author 6".to_string()]},
        Article { id: Uuid::new_v4(), title: query.q, authors: vec!["First author".to_string()]},
    ];
    Json(results)
}

#[post("/add", format = "application/json", data = "<values>")]
fn add(values: Json<AddValues>) -> Json<Article> {
    Json(Article { id: Uuid::new_v4(), title: "Test title".to_string(), authors: vec![]})
}

#[delete("/delete/<id>")]
fn delete(id: UUID) -> Json<Article> {
    Json(Article { id: Uuid::new_v4(), title: "Test title".to_string(), authors: vec![]})
}

#[get("/view/<id>")]
fn view(id: UUID) -> Result<InlineFile, NotFound<String>> {
    let file = NamedFile::open("none").map_err(|_| NotFound(format!("Bad id: {}", id)))?;
    Ok(InlineFile(file))
}

fn main() {
    rocket::ignite()
      .mount("/", routes![index, assets, search, add, delete, view])
      .attach(AdHoc::on_attach(|rocket| {
         let static_dir = rocket.config()
           .get_str("static_dir")
           .unwrap_or("static/")
           .to_string();
         Ok(rocket.manage(StaticDir(static_dir)))
      }))
      .launch();
}
