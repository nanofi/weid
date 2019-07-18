use actix::{Handler, Message};
use failure::Error;
use lmdb::WriteTransaction;

use super::super::article::{Article, ArticleContent};
use super::super::Db;

pub struct Remove(u64);

impl Remove {
  pub fn new(id: u64) -> Self {
    Self(id)
  }
}
impl Message for Remove {
  type Result = Result<Article, Error>;
}

impl Handler<Remove> for Db {
  type Result = Result<Article, Error>;

  fn handle(&mut self, msg: Remove, _: &mut Self::Context) -> Self::Result {
    let key = msg.0;
    let txn = WriteTransaction::new(self.env.clone())?;
    let content = {
      let mut access = txn.access();
      let content = (access.get(&self.db, &key)? as &ArticleContent).to_owned();
      access.del_key(&self.db, &key)?;
      content
    };
    self.id.del(key)?;
    if let Err(e) = self.search.del(key) {
      self.id.add(key)?;
      return Err(e);
    }
    txn.commit()?;
    Ok(Article::new(self.content_path(key), key, content))
  }
}
