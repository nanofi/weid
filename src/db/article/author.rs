
use std::ops::{Index, IndexMut};

use serde::ser::{Serialize, Serializer, SerializeSeq};

const CAP: usize = 128;
const TEXT_CAP: usize = 1024;

#[derive(Copy,Clone)]
pub struct Author {
    len: u16,
    name: [u8; TEXT_CAP],
}

#[derive(Copy,Clone)]
pub struct Authors {
    len: u8,
    arr: [Author; CAP],
}


impl Author {
    pub fn nil() -> Self {
        Author {
            len: 0,
            name: [0; TEXT_CAP],
        }
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
        Authors {
            len: 0,
            arr: [Author::nil(); CAP],
        }
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn push<S: AsRef<str>>(&mut self, author: S) {
        self.arr[self.len as usize].set(author);
        self.len += 1;
    }
}

impl<I : std::slice::SliceIndex<[Author], Output = Author>> Index<I> for Authors {
    type Output = Author;
    fn index(&self, index: I) -> &Self::Output {
        &self.arr[index]
    }
}
impl<I : std::slice::SliceIndex<[Author], Output = Author>> IndexMut<I> for Authors {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
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
