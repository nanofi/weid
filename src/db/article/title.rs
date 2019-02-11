

use serde::ser::{Serialize, Serializer};

const CAP: usize = 65536;

#[derive(Copy,Clone)]
pub struct Title {
  len: u16,
  text: [u8; CAP],
}

impl Title {
  pub fn nil() -> Self {
    Title {
      len: 0,
      text: [0; CAP],
    }
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
