use actix::{Handler, Message};
use failure::Error;
use lmdb::{put, WriteTransaction};
use std::sync::Arc;
use uuid::Uuid;

use super::super::article::{Article, ArticleContent};
use super::super::Db;

pub struct Add {
  title: Arc<str>,
  authors: Arc<[String]>,
}

impl Add {
  pub fn new<S: AsRef<str>, A: AsRef<[String]>>(title: S, authors: A) -> Self {
    Self {
      title: Arc::from(title.as_ref()),
      authors: Arc::from(authors.as_ref()),
    }
  }
}
impl Message for Add {
  type Result = Result<Article, Error>;
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
        info!(
          "Db[Add] Conflict key {}. This is {} time generation.",
          key, times
        );
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
