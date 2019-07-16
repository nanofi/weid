use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use failure::Error;
use uuid::Uuid;

use super::ArticleContent;

pub struct SearchIndex {
  path: PathBuf,
  file: File,
  capacity: u64,
}

impl SearchIndex {
  pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
     std::fs::create_dir_all(path.as_ref())?;
    let file = OpenOptions::new()
      .create(true)
      .read(true)
      .write(true)
      .open(path.as_ref())?;
    let meta = file.metadata()?;
    Ok(Self {
      path: path.as_ref().to_owned(),
      file,
      capacity: meta.len(),
    })
  }

  pub fn add(&mut self, _key: &Uuid, _content: &ArticleContent) -> Result<(), Error> {
    Ok(())
  }

  pub fn del(&mut self, _key: &Uuid) -> Result<(), Error> {
    Ok(())
  }

  pub fn search<S: AsRef<str>>(&self, _words: S) -> Result<Vec<Uuid>, Error> {
    Ok(Vec::new())
  }
}
