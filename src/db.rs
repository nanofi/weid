
use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::ops::{Index, IndexMut};

use serde::ser::{Serialize, Serializer, SerializeSeq};
use uuid::Uuid;

use crate::mmap::{MmappedStruct};

mod errors {
    error_chain! {
        foreign_links {
            Io(std::io::Error);
            Mmap(crate::mmap::Error);
        }
    }
}

use self::errors::*;

const TITLE_TEXT_CAP: usize = 65536;
const AUTHORS_CAP: usize = 128;
const AUTHOR_TEXT_CAP: usize = 1024;

pub struct Title {
    len: u16,
    text: [u8; TITLE_TEXT_CAP],
}

pub struct Author {
    len: u16,
    name: [u8; AUTHOR_TEXT_CAP],
}

pub struct Authors {
    len: u8,
    arr: [Author; AUTHORS_CAP],
}

#[derive(Serialize)]
pub struct Article {
    #[serde(skip)]
    code: u64,
    pub id: Uuid,
    pub title: Title,
    pub authors: Authors,
}

impl Title {
    pub fn nil() -> Self {
        unsafe { Title {
            len: 0,
            text: std::mem::uninitialized(),
        } }
    }

    pub fn to_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.text[..(self.len as usize)]) }
    }

    pub fn set<S: AsRef<str>>(&mut self, val: S) {
        let bytes: &[u8] = val.as_ref().as_ref();
        let len = bytes.len();
        self.len = len as u16;
        (&mut self.text[..len]).copy_from_slice(bytes);
    }
}

impl AsRef<str> for Title {
    fn as_ref(&self) -> &str {
        self.to_str()
    }
}

impl Serialize for Title {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(self.to_str())
    }
}

impl Author {
    pub fn nil() -> Self {
        unsafe { Author {
            len: 0,
            name: std::mem::uninitialized(),
        } }
    }

    pub fn to_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.name[..(self.len as usize)]) }
    }

    pub fn set<S: AsRef<str>>(&mut self, val: S) {
        let bytes: &[u8] = val.as_ref().as_ref();
        let len = bytes.len();
        self.len = len as u16;
        (&mut self.name[..len]).copy_from_slice(bytes);
    }
}

impl AsRef<str> for Author {
    fn as_ref(&self) -> &str {
        self.to_str()
    }
}

impl Serialize for Author {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(self.to_str())
    }
}

impl Authors {
    pub fn nil() -> Self {
        unsafe { Authors {
            len: 0,
            arr: std::mem::uninitialized()
        } }
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn push<S: AsRef<str>>(&mut self, author: S) {
        self.arr[self.len as usize].set(author);
        self.len += 1;
    }
}

impl Index<usize> for Authors {
    type Output = Author;
    fn index(&self, index: usize) -> &Self::Output {
        &self.arr[index]
    }
}
impl IndexMut<usize> for Authors {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.arr[index]
    }
}

impl Serialize for Authors {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(self.len as usize))?;
        for i in 0..(self.len as usize) {
            seq.serialize_element(&self.arr[i])?;
        }
        seq.end()
    }
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
}

#[derive(Default)]
pub struct Metadata {
    articles: u64,
}

impl Metadata {
    fn init<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        if path.as_ref().exists() {
            self.read(path)?;
        } else {
            *self = Default::default();
            self.write(path)?;
        }
        Ok(())
    }
    
    fn read<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&path)?;
        let mut buffer = unsafe { std::slice::from_raw_parts_mut((self as *mut Self) as *mut u8, std::mem::size_of::<Self>()) };
        file.read(&mut buffer)?;
        Ok(())
    }

    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)?;
        let mut buffer = unsafe { std::slice::from_raw_parts((self as *const Self) as *const u8, std::mem::size_of::<Self>()) };
        file.write(&buffer)?;
        Ok(())
    }
}

pub struct DB {
    path: PathBuf,
    metadata: Metadata,
}

impl DB {
    const METADATA_FILE: &'static str = "metadata";
    const DATA_FILE: &'static str = "data";
    const ID_INDEX_FILE: &'static str = "id_index";
    const SEARCH_INDEX_FILE: &'static str = "search_index";

    const CHUNK_SIZE: usize = 1024;

    fn touch<P: AsRef<Path>>(path: P) -> Result<()> {
        OpenOptions::new()
            .write(true)
            .truncate(false)
            .create(true)
            .open(&path)?;
        Ok(())
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<DB> {
        std::fs::create_dir_all(&path)?;

        Self::touch(path.as_ref().join(Self::DATA_FILE))?;
        Self::touch(path.as_ref().join(Self::ID_INDEX_FILE))?;
        Self::touch(path.as_ref().join(Self::SEARCH_INDEX_FILE))?;

        let mut db = unsafe { DB {
            path: path.as_ref().to_owned(),
            metadata: std::mem::uninitialized() ,
        } };

        db.metadata.init(path.as_ref().join(Self::METADATA_FILE))?;

        Ok(db)
    }

    fn add_data(&self, article: &Article) -> Result<()> {
        unimplemented!();
    }

    fn remove_data(&self, index: usize) -> Result<()> {
        unimplemented!();
    }
}

impl Drop for DB {
    fn drop(&mut self) {
        self.metadata.write(self.path.join(Self::METADATA_FILE)).unwrap();
    }
}
