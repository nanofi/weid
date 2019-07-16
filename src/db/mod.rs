mod article;
mod msgs;
mod search;

use std::borrow::Borrow;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::lmdb::{open, Database, DatabaseOptions, EnvBuilder, Environment};
use actix::{Actor, Addr, Arbiter, Context};
use failure::Error;
use uuid::Uuid;

pub use self::article::*;
pub use self::msgs::*;
pub use self::search::*;

pub struct Db {
  path: PathBuf,
  env: Arc<Environment>,
  db: Database<'static>,
  search: SearchIndex,
}

impl Db {
  const DATA_DIR: &'static str = "data";
  const SEARCH_INDEX_DIR: &'static str = "search_index";
  const CONTENT_DIR: &'static str = "content";

  const ADD_LOOP_LIMIT: usize = 10;

  pub fn open<P: AsRef<Path>>(path: P) -> Result<Addr<Self>, Error> {
    let path = path.as_ref().to_owned();
    std::fs::create_dir_all(&path)?;
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

    let search_dir = path.join(Self::SEARCH_INDEX_DIR);
    let search = SearchIndex::open(search_dir)?;

    let arb = Arbiter::new();
    Ok(Self::start_in_arbiter(&arb, |_: &mut Context<Self>| Self {
      path,
      env,
      db,
      search,
    }))
  }

  fn content_path(&self, key: &Uuid) -> PathBuf {
    self
      .path
      .join(Self::CONTENT_DIR)
      .join(format!("{}.pdf", key.to_simple_ref()))
  }
}

impl Actor for Db {
  type Context = Context<Self>;
}
