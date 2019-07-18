use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use failure::Error;

use super::ArticleContent;

pub struct SearchIndex {
  path: PathBuf,
  file: File,
  capacity: u64,
}

impl SearchIndex {
  pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
    unimplemented!();
  }

  pub fn add(&mut self, _key: u64, _content: &ArticleContent) -> Result<(), Error> {
    Ok(())
  }

  pub fn del(&mut self, _key: u64) -> Result<(), Error> {
    Ok(())
  }

  pub fn search<S: AsRef<str>>(&self, _words: S) -> Result<Vec<u64>, Error> {
    Ok(Vec::new())
  }
}
