use std::path::Path;
use std::fs::OpenOptions;
use failure::Error;
use crate::collection::RBTree;

pub struct IdIndex {
  tree: RBTree<u64>,
}
impl IdIndex {
  const FILE_PATH: &'static str = "data";

  pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
    let path = path.as_ref().join(Self::FILE_PATH);
    let file = OpenOptions::new().create(true).write(true).read(true).open(path)?;
    let tree = RBTree::create(file)?;
    Ok(Self { tree })
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
