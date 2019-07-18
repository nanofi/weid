use memmap::MmapMut;
use failure::Error;
use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};

pub struct IdIndex {
  path: PathBuf,
  file: File,
  mmap: MmapMut,
  capacity: u64,
}

impl IdIndex {
  const FILE_PATH: &'static str = "data";
  const LEAST_CAPACITY: u64 = 4096;

  pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
    let path = path.as_ref().join(Self::FILE_PATH);
    let file = OpenOptions::new().create(true).write(true).read(true).open(&path)?;
    let len = file.metadata()?.len();
    let capacity = if len < Self::LEAST_CAPACITY {
      file.set_len(Self::LEAST_CAPACITY)?;
      Self::LEAST_CAPACITY
    } else {
      len
    };
    let mmap = unsafe{ MmapMut::map_mut(&file)? };
    Ok(IdIndex { path, file, mmap, capacity })
  }

  pub fn new(&mut self) -> Result<u64, Error> {
    unimplemented!();
  }

  pub fn add(&mut self, id: u64) -> Result<(), Error> {
    unimplemented!();
  }

  pub fn del(&mut self, id: u64) -> Result<(), Error> {
    unimplemented!();
  }
}
