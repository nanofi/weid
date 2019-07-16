use actix::{Handler, Message};
use failure::Error;
use lmdb::WriteTransaction;
use uuid::Uuid;

use super::super::article::{Article, ArticleContent};
use super::super::Db;

pub struct Remove(Uuid);

impl Remove {
  pub fn new(id: Uuid) -> Self {
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
      let content = (access.get(&self.db, key.as_bytes())? as &ArticleContent).to_owned();
      access.del_key(&self.db, key.as_bytes())?;
      content
    };
    self.search.del(&key)?;
    txn.commit()?;
    Ok(Article::new(self.content_path(&key), key, content))
  }
}
