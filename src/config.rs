
use std::path::PathBuf;

use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug)]
pub struct Config {
  #[serde(default = "Config::default_log")]
  #[serde(deserialize_with = "Config::deserialize_log_level")]
  pub log: simplelog::LevelFilter,
  #[serde(default = "Config::default_port")]
  pub port: u16,
  #[serde(default = "Config::default_address")]
  pub address: String,
  #[serde(default = "Config::default_workers")]
  pub workers: usize,
  #[serde(default = "Config::default_upload_limit")]
  pub upload_limit: usize, 
  #[serde(default = "Config::default_db_path")]
  pub db_path: PathBuf,
}
impl Config {
  fn default_log() -> simplelog::LevelFilter { simplelog::LevelFilter::Off }
  fn default_port() -> u16 { 80 }
  fn default_address() -> String { "localhost".to_owned() }
  fn default_workers() -> usize { num_cpus::get() }
  fn default_upload_limit() -> usize { 1024 * 1024 * 1024 }
  fn default_db_path() -> PathBuf { PathBuf::from("db/") }
  
  fn deserialize_log_level<'de, D>(deserializer: D) -> std::result::Result<simplelog::LevelFilter, D::Error>
  where D: Deserializer<'de> {
    let val = String::deserialize(deserializer)?.to_lowercase();
    let level = match val.as_str() {
      "error" => simplelog::LevelFilter::Error,
      "warn" => simplelog::LevelFilter::Warn,
      "info" => simplelog::LevelFilter::Info,
      "debug" => simplelog::LevelFilter::Debug,
      "trace" => simplelog::LevelFilter::Trace,
      _ => simplelog::LevelFilter::Off,
    };
    Ok(level)
  }
}

impl Default for Config {
  fn default() -> Self {
    Self {
      log: simplelog::LevelFilter::Off,
      port: Self::default_port(),
      address: Self::default_address(),
      workers: Self::default_workers(),
      upload_limit: Self::default_upload_limit(),
      db_path: Self::default_db_path(),
    }
  }
}
