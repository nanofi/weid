use actix::{Handler, Message};
use failure::Error;

use super::super::article::{Article, ArticleContent};
use super::super::Db;

pub struct Search(String);

impl Search {
  pub fn new<S: AsRef<str>>(query: S) -> Self {
    Self(query.as_ref().to_owned())
  }
}
impl Message for Search {
  type Result = Result<Vec<Article>, Error>;
}

impl Handler<Search> for Db {
  type Result = Result<Vec<Article>, Error>;

  fn handle(&mut self, msg: Search, _: &mut Self::Context) -> Self::Result {
    let query = msg.0;
    self.search.search(query)?;

    Ok(Vec::new())
  }
}
