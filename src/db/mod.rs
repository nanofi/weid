
mod article;
mod search;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::borrow::Borrow;

use uuid::Uuid;
use actix::{Context, Actor, Addr, Arbiter, Handler, Message};
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


pub struct Add {
  title: Arc<str>,
  authors: Arc<[String]>,
}
pub struct Remove(Uuid);
pub struct Search(String);
pub struct Get(Uuid);


impl Add {
  pub fn new<S: AsRef<str>, A: AsRef<[String]>>(title: S, authors: A) -> Self {
    Self { title: Arc::from(title.as_ref()), authors: Arc::from(authors.as_ref()) }
  }
}
impl Message for Add {
  type Result = Result<Article, Error>;
}
impl Remove {
  pub fn new(id: Uuid) -> Self {
    Self(id)
  }
}
impl Message for Remove {
  type Result = Result<Article, Error>;
}
impl Search {
  pub fn new<S: AsRef<str>>(query: S) -> Self {
    Self(query.as_ref().to_owned())
  }
}
impl Message for Search {
  type Result = Result<Vec<Article>, Error>;
}
impl Get {
  pub fn new(id: Uuid) -> Self {
    Self(id)
  }
}
impl Message for Get {
  type Result = Result<Article, Error>;
}

impl Db {
  const DATA_DIR: &'static str = "data";
  const SEARCH_INDEX_FILE: &'static str = "search_index";
  const CONTENT_DIR: &'static str = "content";

  const ADD_LOOP_LIMIT: usize = 10;

  pub fn open_in<P: AsRef<Path>>(arb: &Arbiter, path: P) -> Result<Addr<Self>, Error> {
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

    Ok(Self::start_in_arbiter(&arb, |_: &mut Context<Self>| {
      Self { path, env, db, search }
    }))
  }

  fn content_path(&self, key: &Uuid) -> PathBuf {
    self.path.join(Self::CONTENT_DIR).join(format!("{}.pdf", key.to_simple_ref()))
  }
}

impl Actor for Db {
  type Context = Context<Self>;
}

impl Handler<Add> for Db {
  type Result = Result<Article, Error>;
  
  fn handle(&mut self, msg: Add, _: &mut Self::Context) -> Self::Result {
    let content = ArticleContent::new(msg.title, msg.authors);
    let txn = WriteTransaction::new(self.env.clone())?;
    let key = {
      let mut key = Uuid::new_v4();
      let mut access = txn.access();
      let mut times = 0;
      while let Err(e) = access.put(&self.db, key.as_bytes(), &content, put::NOOVERWRITE) {
        info!("Db[Add] Conflict key {}. This is {} time generation.", key, times);
        key = Uuid::new_v4();
        times += 1;
        if times >= Self::ADD_LOOP_LIMIT {
          return Err(format_err!("{:?}", e));
        }
      }
      key
    };
    info!("Db[Add] An article is added with id={}.", key);
    self.search.add(&key, &content)?;
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
    self.search.del(&key)?;
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

    Ok(Vec::new())
  }
}
