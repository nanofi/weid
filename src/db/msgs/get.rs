use actix::{Handler, Message};
use failure::Error;
use lmdb::ReadTransaction;

use super::super::article::{Article, ArticleContent};
use super::super::Db;

pub struct Get(u64);

impl Get {
  pub fn new(id: u64) -> Self {
    Self(id)
  }
}
impl Message for Get {
  type Result = Result<Article, Error>;
}

impl Handler<Get> for Db {
  type Result = Result<Article, Error>;

  fn handle(&mut self, msg: Get, _: &mut Self::Context) -> Self::Result {
    let key = msg.0;
    let txn = ReadTransaction::new(self.env.clone())?;
    let content = (txn.access().get(&self.db, &key)? as &ArticleContent).to_owned();
    Ok(Article::new(self.content_path(key), key, content))
  }
}
