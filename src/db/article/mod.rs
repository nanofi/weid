mod author;
mod title;

use std::path::{Path, PathBuf};

use crate::lmdb::traits::LmdbRaw;
use serde::{ser::SerializeStruct, Serialize, Serializer};

pub use self::author::*;
pub use self::title::*;

#[derive(Serialize, Copy, Clone)]
pub struct ArticleContent {
  pub title: Title,
  pub authors: Authors,
}

unsafe impl LmdbRaw for ArticleContent {}

impl ArticleContent {
  pub fn new<T: AsRef<str>, I: AsRef<str>, A: AsRef<[I]>>(title: T, authors: A) -> Self {
    ArticleContent {
      title: Title::new(title),
      authors: Authors::new(authors),
    }
  }
}

pub struct Article {
  path: PathBuf,
  id: u64,
  content: ArticleContent,
}

impl Serialize for Article {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut s = serializer.serialize_struct("Article", 3)?;
    s.serialize_field("id", &self.id)?;
    s.serialize_field("title", &self.content.title)?;
    s.serialize_field("authors", &self.content.authors)?;
    s.end()
  }
}

impl Article {
  pub fn new(path: PathBuf, id: u64, content: ArticleContent) -> Self {
    Self { path, id, content }
  }

  pub fn path(&self) -> &Path {
    self.path.as_path()
  }

  pub fn filename(&self) -> String {
    self
      .path
      .file_name()
      .map_or(String::new(), |s| s.to_string_lossy().to_string())
  }
}
