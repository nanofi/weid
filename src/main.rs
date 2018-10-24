#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(custom_derive)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate url;
extern crate url_serde;
extern crate uuid;
#[macro_use] extern crate error_chain;
extern crate memmap;

mod db;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use rocket::fairing::AdHoc;
use rocket::State;
use rocket::request::Request;
use rocket::response::{self, Response, Responder, NamedFile};
use rocket::response::status::NotFound;

use rocket_contrib::{Json, UUID};

use url::Url;

use self::db::{Article, DB};

struct ExtraOptions {
    static_dir: String,
}

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
fn assets(path: PathBuf, options: State<ExtraOptions>) -> Option<NamedFile> {
    NamedFile::open(Path::new(&options.static_dir).join(path)).ok()
}

#[get("/search?<query>")]
fn search(query: Query) -> Json<Vec<Article>> {
    let results = vec![];
    Json(results)
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

fn main() {
    rocket::ignite()
        .mount("/", routes![index, assets, search, add, delete, view])
        .attach(AdHoc::on_attach(|rocket| {
            let extra_options = ExtraOptions {
                static_dir: rocket.config().get_str("static_dir").unwrap_or("static/").to_owned(),
            };
          Ok(rocket.manage(extra_options))
        }))
        .attach(AdHoc::on_attach(|rocket| {
            let path = Path::new(rocket.config().get_str("db_path").unwrap_or("db/"));
            if let Ok(db) = DB::open(path) {
                Ok(rocket.manage(Arc::new(db)))
            } else {
                Err(rocket)
            }
        }))
        .launch();
}
