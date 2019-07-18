mod article;
mod id;
mod msgs;
mod search;

use std::borrow::Borrow;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::lmdb::{open, Database, DatabaseOptions, EnvBuilder, Environment};
use actix::{Actor, Addr, Arbiter, Context};
use failure::Error;

pub use self::article::*;
use self::id::*;
pub use self::msgs::*;
use self::search::*;

pub struct Db {
  path: PathBuf,
  env: Arc<Environment>,
  db: Database<'static>,
  id: IdIndex,
  search: SearchIndex,
}

impl Db {
  const DATA_DIR: &'static str = "data";
  const INDEX_DIR: &'static str = "index";
  const CONTENT_DIR: &'static str = "content";

  pub fn open<P: AsRef<Path>>(path: P) -> Result<Addr<Self>, Error> {
    let path = path.as_ref().to_owned();
    std::fs::create_dir_all(path.join(Self::CONTENT_DIR))?;

    let data_dir = path.join(Self::DATA_DIR);
    std::fs::create_dir_all(&data_dir)?;
    let env = Arc::new(unsafe {
      EnvBuilder::new()?.open(
        data_dir.to_string_lossy().borrow(),
        open::Flags::empty(),
        0o600,
      )?
    });
    let db = Database::open(env.clone(), None, &DatabaseOptions::defaults())?;

    let index_dir = path.join(Self::INDEX_DIR);
    std::fs::create_dir_all(&index_dir)?;
    let id = IdIndex::open(&index_dir)?;
    let search = SearchIndex::open(&index_dir)?;

    let arb = Arbiter::new();
    Ok(Self::start_in_arbiter(&arb, |_: &mut Context<Self>| Self {
      path,
      env,
      db,
      id,
      search,
    }))
  }

  fn content_path(&self, key: u64) -> PathBuf {
    self
      .path
      .join(Self::CONTENT_DIR)
      .join(format!("{}.pdf", key))
  }
}

impl Actor for Db {
  type Context = Context<Self>;
}
