
use std::marker::PhantomData;
use std::path::Path;
use std::fs::{File, OpenOptions};

mod errors {
    error_chain! {
        foreign_links {
            Io(std::io::Error);
        }
    }
}

pub use self::errors::*;

pub struct MmappedStruct<T> {
    file: File,
    struct_type: PhantomData<T>,
}

impl<T> MmappedStruct<T> {
    const TYPE_SIZE: usize = std::mem::size_of::<T>();

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(false)
            .create(true)
            .open(&path)?;
        file.set_len(Self::TYPE_SIZE as u64)?;
        Ok(Self {
            file: file,
            struct_type: PhantomData,
        })
    }
}
