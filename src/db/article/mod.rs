
mod title;
mod author;

use std::path::{Path, PathBuf};
use std::fs::{OpenOptions, File};

use uuid::Uuid;
use crate::lmdb::traits::LmdbRaw;

pub use self::title::*;
pub use self::author::*;

#[derive(Serialize,Copy,Clone)]
pub struct ArticleContent {
  pub title: Title,
  pub authors: Authors,
}

unsafe impl LmdbRaw for ArticleContent {}

#[derive(Serialize)]
pub struct Article {
}


impl Article {
  pub fn nil() -> Self {
    Self {}
  }
  
  pub fn path(&self) -> PathBuf {
    unimplemented!();
  }
  
  pub fn filename(&self) -> String {
    unimplemented!();
  }
}
