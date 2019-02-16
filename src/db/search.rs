
use std::path::{Path, PathBuf};

use uuid::Uuid;
use failure::Error;

use super::ArticleContent;

pub struct SearchIndex {
  path: PathBuf,
}

impl SearchIndex {
  pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
    let path = path.as_ref().to_owned();
    Ok(Self { path })
  }

  pub fn add(&self, key: &Uuid, content: &ArticleContent) -> Result<(), Error> {
    Ok(())
  }

  pub fn del(&self, key: &Uuid) -> Result<(), Error> {
    Ok(())
  }

  pub fn search<S: AsRef<str>>(&self, words: S) -> Result<Vec<Uuid>, Error> {
    Ok(Vec::new())
  }
}
