
mod errors;
mod article;

use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use uuid::Uuid;
use actix::{Context, Actor, Addr};

pub use self::errors::*;
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
#[rtype(result="Result<Article>")]
pub struct Add(AddParams);
#[derive(Message)]
#[rtype(result="Result<Article>")]
pub struct Remove(Uuid);
#[derive(Message)]
#[rtype(result="Result<Vec<Article>>")]
pub struct Search(String);
#[derive(Message)]
#[rtype(result="Result<Article>")]
pub struct Get(Uuid);

pub trait DbOperators {
    fn add(params: AddParams) -> Result<Article>;
    fn remove(id: Uuid) -> Result<Article>;
    fn search<S: AsRef<str>>(query: S) -> Result<Vec<Article>>;
    fn get(id: Uuid) -> Result<Article>;
}

impl Db {
    const DATA_FILE: &'static str = "data";
    const ID_INDEX_FILE: &'static str = "id_index";
    const SEARCH_INDEX_FILE: &'static str = "search_index";

    const CHUNK_SIZE: usize = 1024;

    fn touch<P: AsRef<Path>>(path: P) -> Result<()> {
        OpenOptions::new()
            .write(true)
            .truncate(false)
            .create(true)
            .open(&path)?;
        Ok(())
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Addr<Self>> {
        let path = path.as_ref().to_owned();
        std::fs::create_dir_all(&path)?;

        Self::touch(path.join(Self::DATA_FILE))?;
        Self::touch(path.join(Self::ID_INDEX_FILE))?;
        Self::touch(path.join(Self::SEARCH_INDEX_FILE))?;

        Ok(Self::create(move |ctx: &mut Context<Self>| Self {
            path: path
        }))
    }
}

impl Actor for Db {
    type Context = Context<Self>;
}

impl DbOperators for Addr<Db> {
    fn add(params: AddParams) -> Result<Article> {
        unimplemented!();
    }
    fn remove(id: Uuid) -> Result<Article> {
        unimplemented!();
    }
    fn search<S: AsRef<str>>(query: S) -> Result<Vec<Article>> {
        unimplemented!();
    }
    fn get(id: Uuid) -> Result<Article> {
        unimplemented!();
    }
}
