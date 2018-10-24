
use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::ops::{Index, IndexMut};
use std::marker::PhantomData;

use serde::ser::{Serialize, Serializer, SerializeSeq};
use uuid::Uuid;
use memmap::{MmapMut};

mod errors {
    error_chain! {
        foreign_links {
            Io(std::io::Error);
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


struct TypedMmapMut<T> {
    mmap: MmapMut,
    mtype: PhantomData<T>,
}

struct MmappedFile<T> {
    file: File,
    memory_type: PhantomData<T>,
}

impl<T> MmappedFile<T> {
    fn open_with_len<P: AsRef<Path>>(path: P, len: usize) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(true)
            .open(path)?;
        file.set_len((len * std::mem::size_of::<T>()) as u64)?;
        Ok(Self {
            file: file,
            memory_type: PhantomData,
        })
    }
}

struct Metadata {
    articles: u64,
}

pub struct DB {
    path: PathBuf,
    metadata: MmappedFile<Metadata>,
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

        Ok(DB {
            path: path.as_ref().to_owned(),
            metadata: MmappedFile::open_with_len(path.as_ref().join(Self::METADATA_FILE), 1)?
        })
    }

    fn add_data(&self, article: &Article) -> Result<()> {
        Ok(())
    }

    fn remove_data(&self, index: usize) -> Result<()> {
        Ok(())
    }
}
