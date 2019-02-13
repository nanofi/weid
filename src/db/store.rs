
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::borrow::Borrow;

use uuid::Uuid;
use actix::{Context, Actor, Addr, Arbiter, Handler};
use failure::Error;

use crate::lmdb::{open, put, EnvBuilder, Environment, WriteTransaction, Database, DatabaseOptions};

use super::article::*;

#[derive(Message)]
#[rtype(result="Result<Uuid, Error>")]
pub struct Add(ArticleContent);
#[derive(Message)]
#[rtype(result="Result<Uuid, Error>")]
pub struct Remove(Uuid);

impl Add {
  pub fn new(article: ArticleContent) -> Self {
    Self(article)
  }
}

pub struct Store {
  path: PathBuf,
  env: Arc<Environment>,
  db: Database<'static>,
}

impl Store {
  pub fn open<P: AsRef<Path>>(path: P) -> Result<Addr<Self>, Error> {
    let path = path.as_ref().to_owned();
    std::fs::create_dir_all(&path)?;
    let env = Arc::new(unsafe {
      EnvBuilder::new()?
        .open(path.to_string_lossy().borrow(), open::Flags::empty(), 0o600)?
    });
    let db = Database::open(env.clone(), None, &DatabaseOptions::defaults())?;
    Ok(Arbiter::start(|_: &mut Context<Self>| {
      Self {
        path: path,
        env: env,
        db: db,
      }
    }))
  }
}


impl Actor for Store {
  type Context = Context<Self>;
}


impl Handler<Add> for Store {
  type Result = Result<Uuid, Error>;
  
  fn handle(&mut self, msg: Add, ctx: &mut Self::Context) -> Self::Result {
    let mut key = Uuid::new_v4();
    {
      let txn = WriteTransaction::new(self.env.clone())?;
      while let Err(_) = txn.access().put(&self.db, key.as_bytes(), &msg.0, put::NOOVERWRITE) {
        key = Uuid::new_v4();
      }
      txn.commit()?;
    }
    Ok(key)
  }
}
