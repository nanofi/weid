use actix::{Handler, Message};
use failure::Error;
use lmdb::{put, WriteTransaction};
use std::sync::Arc;

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
    let key = self.id.new()?;
    {
      let mut access = txn.access();
      if let Err(e) = access.put(&self.db, &key, &content, put::NOOVERWRITE) {
        return Err(format_err!("{:?}", e));
      }
      info!("Db[Add] An article is added with id={}.", key);
      self.search.add(key, &content)?;
    }
    txn.commit()?;
    Ok(Article::new(self.content_path(key), key, content))
  }
}
