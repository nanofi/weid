
mod article;

use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use uuid::Uuid;
use futures::Future;
use actix::{msgs, Context, Actor, Addr, Arbiter, Handler};
use failure::Error;

pub use self::article::*;


pub struct Db {
    path: PathBuf,
}

pub struct AddParams {
    pub title: String,
    pub authors: Vec<String>,
    pub file: Vec<u8>,
}

#[derive(Message)]
#[rtype(result="Result<Article, Error>")]
pub struct Add(AddParams);
#[derive(Message)]
#[rtype(result="Result<Article, Error>")]
pub struct Remove(Uuid);
#[derive(Message)]
#[rtype(result="Result<Vec<Article>, Error>")]
pub struct Search(String);
#[derive(Message)]
#[rtype(result="Result<Article, Error>")]
pub struct Get(Uuid);

impl Add {
    pub fn new(title: String, authors: Vec<String>, file: Vec<u8>) -> Self {
        Self(AddParams{ title, authors, file })
    }
}
impl Remove {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }
}
impl Search {
    pub fn new<S: AsRef<str>>(query: S) -> Self {
        Self(query.as_ref().to_owned())
    }
}
impl Get {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }
}

impl Db {
    const DATA_FILE: &'static str = "data";
    const ID_INDEX_FILE: &'static str = "id_index";
    const SEARCH_INDEX_FILE: &'static str = "search_index";

    const CHUNK_SIZE: usize = 1024;

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Addr<Self>, Error> {
        let path = path.as_ref().to_owned();
        std::fs::create_dir_all(&path)?;
            
        let arbiter = Arbiter::new("db");

        let addr = arbiter.send(msgs::StartActor::new(|_: &mut Context<Self>| {
            Self {
                path: path
            }
        })).wait()?;
        Ok(addr)
    }
}

impl Actor for Db {
    type Context = Context<Self>;
}

impl Handler<Add> for Db {
    type Result = Result<Article, Error>;

    fn handle(&mut self, msg: Add, ctx: &mut Self::Context) -> Self::Result {
        Ok(Article::nil())
    }
}

impl Handler<Remove> for Db {
    type Result = Result<Article, Error>;

    fn handle(&mut self, msg: Remove, ctx: &mut Self::Context) -> Self::Result {
        Ok(Article::nil())
    }
}

impl Handler<Get> for Db {
    type Result = Result<Article, Error>;

    fn handle(&mut self, msg: Get, ctx: &mut Self::Context) -> Self::Result {
        Ok(Article::nil())
    }
}

impl Handler<Search> for Db {
    type Result = Result<Vec<Article>, Error>;

    fn handle(&mut self, msg: Search, ctx: &mut Self::Context) -> Self::Result {
        Ok(vec![])
    }
}
