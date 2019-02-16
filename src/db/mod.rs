
mod article;
mod search;

use std::path::{Path, PathBuf};
use std::fs::{remove_file, OpenOptions};
use std::io::Write;
use std::sync::Arc;
use std::borrow::Borrow;

use uuid::Uuid;
use actix::{Context, Actor, Addr, Arbiter, Handler};
use crate::lmdb::{open, put, EnvBuilder, Environment, WriteTransaction, ReadTransaction, Database, DatabaseOptions};
use failure::Error;

pub use self::article::*;
pub use self::search::*;

pub struct Db {
  path: PathBuf,
  env: Arc<Environment>,
  db: Database<'static>,
  search: SearchIndex,
}

#[derive(Message)]
#[rtype(result="Result<Article, Error>")]
pub struct Add {
  pub title: String,
  pub authors: Vec<String>,
  pub file: Vec<u8>,
}
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
    Self{ title, authors, file }
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
  const DATA_DIR: &'static str = "data";
  const SEARCH_INDEX_FILE: &'static str = "search_index";
  const CONTENT_DIR: &'static str = "content";
  
  pub fn open<P: AsRef<Path>>(path: P) -> Result<Addr<Self>, Error> {
    let path = path.as_ref().to_owned();
    std::fs::create_dir_all(&path)?;
    std::fs::create_dir_all(path.join(Self::CONTENT_DIR))?;

    let data_dir = path.join(Self::DATA_DIR);
    std::fs::create_dir_all(&data_dir)?;
    let env = Arc::new(unsafe {
      EnvBuilder::new()?.open(data_dir.to_string_lossy().borrow(), open::Flags::empty(), 0o600)?
    });
    let db = Database::open(env.clone(), None, &DatabaseOptions::defaults())?;
    let search = SearchIndex::open(path.join(Self::SEARCH_INDEX_FILE))?;
    
    Ok(Arbiter::start(|_: &mut Context<Self>| {
      Self { path, env, db, search }
    }))
  }

  fn content_path(&self, key: &Uuid) -> PathBuf {
    self.path.join(Self::CONTENT_DIR).join(key.to_simple_ref().to_string())
  }
}

impl Actor for Db {
  type Context = Context<Self>;
}

impl Handler<Add> for Db {
  type Result = Result<Article, Error>;
  
  fn handle(&mut self, msg: Add, _: &mut Self::Context) -> Self::Result {
    let content = ArticleContent::new(msg.title, msg.authors);
    let mut key = Uuid::new_v4();
    let txn = WriteTransaction::new(self.env.clone())?;
    {
      let mut access = txn.access();
      while let Err(_) = access.put(&self.db, key.as_bytes(), &content, put::NOOVERWRITE) {
        key = Uuid::new_v4();
      }
    }
    {
      let path = self.content_path(&key);
      let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(path)?;
      file.write(&msg.file)?;
      file.flush()?;
    }
    {
      self.search.add(&key, &content)?;
    }
    txn.commit()?;
    Ok(Article::new(self.content_path(&key), key, content))
  }
}

impl Handler<Remove> for Db {
  type Result = Result<Article, Error>;
  
  fn handle(&mut self, msg: Remove, _: &mut Self::Context) -> Self::Result {
    let key = msg.0;
    let txn = WriteTransaction::new(self.env.clone())?;
    let content = {
      let mut access = txn.access();
      let content = (access.get(&self.db, key.as_bytes())? as &ArticleContent).to_owned();
      access.del_key(&self.db, key.as_bytes())?;
      content
    };
    {
      let path = self.content_path(&key);
      remove_file(&path)?;
    }
    {
      self.search.del(&key)?;
    }
    txn.commit()?;
    Ok(Article::new(self.content_path(&key), key, content))
  }
}

impl Handler<Get> for Db {
  type Result = Result<Article, Error>;

  fn handle(&mut self, msg: Get, _: &mut Self::Context) -> Self::Result {
    let key = msg.0;
    let txn = ReadTransaction::new(self.env.clone())?;
    let content = (txn.access().get(&self.db, key.as_bytes())? as &ArticleContent).to_owned();
    Ok(Article::new(self.content_path(&key), key, content))
  }
}

impl Handler<Search> for Db {
  type Result = Result<Vec<Article>, Error>;
  
  fn handle(&mut self, msg: Search, _: &mut Self::Context) -> Self::Result {
    let query = msg.0;
    self.search.search(query)?;
    
    unimplemented!();
  }
}
