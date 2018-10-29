
mod title;
mod author;

use std::path::{Path, PathBuf};
use std::fs::{OpenOptions, File};

use uuid::Uuid;

pub use self::title::*;
pub use self::author::*;

#[derive(Serialize,Copy,Clone)]
pub struct Article {
    #[serde(skip)]
    code: u64,
    pub id: Uuid,
    pub title: Title,
    pub authors: Authors,
}

impl Article {
    pub fn nil() -> Self {
        Article {
            code: 0,
            id: Uuid::nil(),
            title: Title::nil(),
            authors: Authors::nil(),
        }
    }

    pub fn id_str(&self) -> String {
        let mut id = String::new();
        std::fmt::write(&mut id, format_args!("{}", self.id.to_simple_ref())).expect("This should not be occurd");
        id
    }

    pub fn path<P: AsRef<Path>>(&self, base: P) -> PathBuf {
        base.as_ref().join(self.id_str())
    }

    pub fn filename(&self) -> String {
        let mut f = String::from(self.title.to_str());
        f.push_str(".pdf");
        f
    }
}
