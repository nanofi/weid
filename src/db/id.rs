use memmap::MmapMut;
use failure::Error;
use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::cmp::Ordering;

#[repr(C)]
struct Node {
  color: bool,
  parent: Option<u64>,
  left: Option<u64>,
  right: Option<u64>,
  val: u64,
}

impl Node {
  fn zero() -> Self {
    Node {
      color: false,
      parent: None,
      left: None,
      right: None,
      val: 0u64,
    }
  }

  #[inline]
  fn is_red(&self) -> bool {
    self.color
  }
  #[inline]
  fn is_black(&self) -> bool {
    !self.color
  }
  #[inline]
  fn to_red(&mut self) {
    self.color = true;
  }
  #[inline]
  fn to_black(&mut self) {
    self.color = false;
  }
}

struct Mem(MmapMut);

impl Mem {
  #[inline]
  fn len(&self) -> usize {
    unsafe{ *(self.0.as_ptr() as *const u64) as usize }
  }

  #[inline]
  fn occupy(&self) -> usize {
    mem::size_of::<u64>() + mem::size_of::<Option<u64>>() + mem::size_of::<Node>() * self.len()
  }

  #[inline]
  fn new(&mut self) -> u64 {
    let len = self.len();
    unsafe {
      *(self.0.as_ptr() as *mut u64) += 1; 
    }
    len as u64
  }

  #[inline]
  fn root(&self) -> Option<u64> {
    unsafe { *(self.0.as_ptr().add(mem::size_of::<u64>()) as *const Option<u64>) }
  }
  #[inline]
  fn set_root(&mut self, val: Option<u64>) {
    unsafe { *(self.0.as_mut_ptr().add(mem::size_of::<u64>()) as *mut Option<u64>) = val; }
  }
}

impl Deref for Mem {
  type Target = [Node];
  #[inline]
  fn deref(&self) -> &Self::Target {
    unsafe { 
      std::slice::from_raw_parts(
        self.0.as_ptr()
          .add(mem::size_of::<u64>())
          .add(mem::size_of::<Option<u64>>()) as *const Node, self.len()) 
    }
  }
}
impl DerefMut for Mem {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { 
      std::slice::from_raw_parts_mut(
        self.0.as_mut_ptr()
          .add(mem::size_of::<u64>())
          .add(mem::size_of::<Option<u64>>()) as *mut Node, self.len()) 
    }
  }
}
impl AsRef<[Node]> for Mem {
  #[inline]
  fn as_ref(&self) -> &[Node] {
    self.deref()
  }
}
impl AsMut<[Node]> for Mem {
  #[inline]
  fn as_mut(&mut self) -> &mut [Node] {
    self.deref_mut()
  }
}

pub struct IdIndex {
  file: File,
  mem: Mem,
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
    let mem = Mem(unsafe{ MmapMut::map_mut(&file)? });
    Ok(IdIndex { file, mem, capacity })
  }

  fn extend(&mut self) -> Result<(), Error> {
    self.capacity *= 2;
    self.file.set_len(self.capacity)?;
    self.mem = Mem(unsafe{ MmapMut::map_mut(&self.file)? });
    Ok(())
  }

  fn shrink(&mut self) -> Result<(), Error> {
    self.capacity /= 2;
    self.file.set_len(self.capacity)?;
    self.mem = Mem(unsafe{ MmapMut::map_mut(&self.file)? });
    Ok(())
  }

  fn new_node(&mut self) -> Result<u64, Error> {
    if self.mem.occupy() + mem::size_of::<Node>() > self.capacity as usize {
      self.extend()?;
    }
    let n = self.mem.new();
    self.mem[n as usize] = Node::zero();
    Ok(n)
  }

  fn del_node(&mut self, x: u64) -> Result<(), Error> {
    let last = self.mem.len()-1;
    self.mem.swap(x as usize, last);
    if let Some(i) = self.mem[x as usize].parent {
      if Some(last as u64) == self.mem[i as usize].left {
        self.mem[i as usize].left = Some(x)
      } else {
        self.mem[i as usize].right = Some(x);
      }
    } else {
      self.mem.set_root(Some(x));
    }
    if let Some(i) = self.mem[x as usize].left {
      self.mem[i as usize].parent = Some(x);
    }
    if let Some(i) = self.mem[x as usize].right {
      self.mem[i as usize].parent = Some(x);
    }
    self.mem[last] = Node::zero();
    if self.mem.occupy() < (self.capacity as usize)/2 {
      self.shrink()?;
    }
    Ok(())
  }

  pub fn new(&mut self) -> Result<u64, Error> {
    unimplemented!();
  }

  #[inline]
  fn swap_color(&mut self, x: usize, y: usize) {
    let c = self.mem[x].color;
    self.mem[x].color = self.mem[y].color;
    self.mem[y].color = c;
  }

  fn rotate_left(&mut self, x: u64) {
    let rc = self.mem[x as usize].right.unwrap();
    let rcl = self.mem[rc as usize].left;
    self.mem[x as usize].right = rcl;
    if let Some(i) = rcl {
      self.mem[i as usize].parent = Some(x);
    }
    self.mem[rc as usize].parent = self.mem[x as usize].parent;
    if let Some(p) = self.mem[x as usize].parent {
      if Some(x) == self.mem[p as usize].left {
        self.mem[p as usize].left = Some(rc);
      } else {
        self.mem[p as usize].right = Some(rc);
      }
    } else {
      self.mem.set_root(Some(rc));
    }
    self.mem[rc as usize].left = Some(x);
    self.mem[x as usize].parent = Some(rc);
  }

  fn rotate_right(&mut self, x: u64) {
    let lc = self.mem[x as usize].left.unwrap();
    let lcr = self.mem[lc as usize].right;
    self.mem[x as usize].left = lcr;
    if let Some(i) = lcr {
      self.mem[i as usize].parent = Some(x);
    }
    self.mem[lc as usize].parent = self.mem[x as usize].parent;
    if let Some(p) = self.mem[x as usize].parent {
      if Some(x) == self.mem[p as usize].left {
        self.mem[p as usize].left = Some(lc);
      } else {
        self.mem[p as usize].right = Some(lc);
      }
    } else {
      self.mem.set_root(Some(lc));
    }
    self.mem[lc as usize].right = Some(x);
    self.mem[x as usize].parent = Some(lc);
  }

  pub fn add(&mut self, id: u64) -> Result<(), Error> {
    let mut x = self.mem.root();
    let mut p = None;
    while x.is_some() {
      let i = x.unwrap() as usize;
      p = x;
      if id < self.mem[i].val {
        x = self.mem[i].left;
      } else {
        x = self.mem[i].right;
      }
    }
    let node = self.new_node()?;
    self.mem[node as usize].parent = p;
    self.mem[node as usize].to_red();
    if let Some(p) = p {
      if id < self.mem[p as usize].val {
        self.mem[p as usize].left = Some(node);
      } else {
        self.mem[p as usize].right = Some(node);
      }
    } else {
      self.mem.set_root(Some(node));
    }
    let mut x = node;
    while x != self.mem.root().unwrap() && self.mem[self.mem[x as usize].parent.unwrap() as usize].is_red() {
      let p = self.mem[x as usize].parent.unwrap();
      let g = self.mem[p as usize].parent.unwrap();
      if Some(p) == self.mem[g as usize].left {
        let u = self.mem[g as usize].right;
        if u.is_some() && self.mem[u.unwrap() as usize].is_red() {
          let u = u.unwrap();
          self.mem[u as usize].to_black();
          self.mem[p as usize].to_black();
          self.mem[g as usize].to_red();
          x = g;
        } else {
          if Some(x) == self.mem[p as usize].right {
            self.rotate_left(p);
            x = g;
          } else {
            self.swap_color(p as usize, g as usize);
            x = p;
          }
          self.rotate_right(g);
        }
      } else {
        let u = self.mem[g as usize].left;
        if u.is_some() && self.mem[u.unwrap() as usize].is_red() {
          let u = u.unwrap();
          self.mem[u as usize].to_black();
          self.mem[p as usize].to_black();
          self.mem[g as usize].to_red();
          x = g;
        } else {
          if Some(x) == self.mem[p as usize].left {
            self.rotate_right(p);
            x = g;
          } else {
            self.swap_color(p as usize, g as usize);
            x = p;
          }
          self.rotate_left(g);
        }
      }
    }
    let r = self.mem.root().unwrap();
    self.mem[r as usize].to_black();
    Ok(())
  }

  fn del_bst(&mut self, x: Option<u64>, id: u64) -> Option<u64> {
    let x = match x {
      None => return None,
      Some(x) => x,
    };
    match id.cmp(&self.mem[x as usize].val) {
      Ordering::Less => self.del_bst(self.mem[x as usize].left, id),
      Ordering::Greater => self.del_bst(self.mem[x as usize].right, id),
      Ordering::Equal => {
        if let (Some(_), Some(r)) = (self.mem[x as usize].left, self.mem[x as usize].right) {
          let mut m = r;
          while let Some(n) = self.mem[m as usize].left {
            m = n;
          }
          let v = self.mem[m as usize].val;
          self.mem[x as usize].val = v; 
          self.del_bst(Some(r), v)
        } else {
          Some(x)
        }
      }
    }
  }

  fn is_black(&self, x: Option<u64>) -> bool {
    x.is_none() || self.mem[x.unwrap() as usize].is_black()
  }

  fn del_dblack(&mut self, p: Option<u64>, x: Option<u64>) {
    let p = match p {
      None => return,
      Some(p) => p,
    };
    if x == self.mem[p as usize].left {
      let mut s = self.mem[p as usize].right.unwrap();
      if self.mem[s as usize].is_red() {
        self.mem[s as usize].to_black();
        self.mem[p as usize].to_red();
        self.rotate_left(s);
        s = self.mem[p as usize].right.unwrap();
      }
      let l = self.mem[s as usize].left;
      let r = self.mem[s as usize].right;
      if self.is_black(l) && self.is_black(r) {
        self.mem[s as usize].to_red();
        if self.mem[p as usize].is_red() {
          self.mem[p as usize].to_black();
        } else {
          self.del_dblack(self.mem[p as usize].parent, Some(p));
        }
      } else {
        if self.is_black(r) {
          self.mem[l.unwrap() as usize].to_black();
          self.mem[s as usize].to_red();
          self.rotate_right(s);
          s = self.mem[p as usize].right.unwrap();
        }
        self.mem[s as usize].color = self.mem[p as usize].color;
        self.mem[p as usize].to_black();
        let r = self.mem[s as usize].right.unwrap();
        self.mem[r as usize].to_black();
        self.rotate_left(p);
      }
    } else {
      let mut s = self.mem[p as usize].left.unwrap();
      if self.mem[s as usize].is_red() {
        self.mem[s as usize].to_black();
        self.mem[p as usize].to_red();
        self.rotate_right(s);
        s = self.mem[p as usize].left.unwrap();
      }
      let l = self.mem[s as usize].left;
      let r = self.mem[s as usize].right;
      if self.is_black(l) && self.is_black(r) {
        self.mem[s as usize].to_red();
        if self.mem[p as usize].is_red() {
          self.mem[p as usize].to_black();
        } else {
          self.del_dblack(self.mem[p as usize].parent, Some(p));
        }
      } else {
        if self.is_black(l) {
          self.mem[r.unwrap() as usize].to_black();
          self.mem[s as usize].to_red();
          self.rotate_right(s);
          s = self.mem[p as usize].left.unwrap();
        }
        self.mem[s as usize].color = self.mem[p as usize].color;
        self.mem[p as usize].to_black();
        let l = self.mem[s as usize].left.unwrap();
        self.mem[l as usize].to_black();
        self.rotate_right(p);
      }
    }
  }

  pub fn del(&mut self, id: u64) -> Result<(), Error> {
    let x = self.del_bst(self.mem.root(), id);
    let x = match x {
      None => return Ok(()),
      Some(x) if Some(x) == self.mem.root() => {
        self.mem.set_root(None);
        return Ok(());
      },
      Some(x) => x,
    };
    let c = self.mem[x as usize].left.and(self.mem[x as usize].right);
    if self.mem[x as usize].is_red() || (c.is_some() && self.mem[c.unwrap() as usize].is_red()) {
      let p = self.mem[x as usize].parent.unwrap();
      if Some(x) == self.mem[p as usize].left {
        self.mem[p as usize].left = c;
      } else {
        self.mem[p as usize].right = c;
      }
      if let Some(c) = c {
        self.mem[c as usize].to_black();
        self.mem[c as usize].parent = Some(p);
      }
    } else {
      let p = self.mem[x as usize].parent;
      if let Some(p) = p {
        if Some(x) == self.mem[p as usize].left {
          self.mem[p as usize].left = c
        } else {
          self.mem[p as usize].right = c
        }
      } else {
        self.mem.set_root(c);
      }
      if let Some(c) = c {
        self.mem[c as usize].parent = p;
      }
      self.del_dblack(p, c);
    }
    self.del_node(x)?;
    let r = self.mem.root().unwrap();
    self.mem[r as usize].to_black();
    Ok(())
  }
}
