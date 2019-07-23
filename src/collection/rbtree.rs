use memmap::MmapMut;
use failure::Error;
use std::path::Path;
use std::fs::{File, OpenOptions};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;
use std::cmp::{Ord, Ordering};

#[repr(C)]
struct Node<T> {
  color: bool,
  parent: Option<u64>,
  left: Option<u64>,
  right: Option<u64>,
  val: T,
}

impl<T: Default> Node<T> {
  fn zero() -> Self {
    Self {
      color: false,
      parent: None,
      left: None,
      right: None,
      val: Default::default(),
    }
  }
}

impl<T> Node<T> {
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

struct Mem<T> {
  mmap: MmapMut,
  phantom: PhantomData<T>,
}

impl<T> Mem<T> {
  fn new(mmap: MmapMut) -> Self {
    Self { mmap, phantom: PhantomData }
  }

  #[inline]
  fn len(&self) -> u64 {
    unsafe{ *(self.mmap.as_ptr() as *const u64) }
  }
  #[inline]
  fn len_mut(&mut self) -> &mut u64 {
    unsafe { &mut *(self.mmap.as_mut_ptr() as *mut u64) }
  }
  #[inline]
  fn root(&self) -> Option<u64> {
    unsafe { *(self.mmap.as_ptr().add(mem::size_of::<u64>()) as *const Option<u64>) }
  }
  #[inline]
  fn root_mut(&mut self) -> &mut Option<u64> {
    unsafe { &mut *(self.mmap.as_mut_ptr().add(mem::size_of::<u64>()) as *mut Option<u64>) }
  }
  
  #[inline]
  fn occupy(&self) -> usize {
    mem::size_of::<u64>() + mem::size_of::<Option<u64>>() + mem::size_of::<Node<T>>() * self.len() as usize
  }

  #[inline]
  fn push(&mut self) -> u64 {
    let len = self.len();
    *self.len_mut() += 1;
    len
  }

  #[inline]
  fn set_root(&mut self, val: Option<u64>) {
    *self.root_mut() = val;
  }
}

impl<T> Deref for Mem<T> {
  type Target = [Node<T>];
  #[inline]
  fn deref(&self) -> &Self::Target {
    unsafe { 
      std::slice::from_raw_parts(
        self.mmap.as_ptr()
          .add(mem::size_of::<u64>())
          .add(mem::size_of::<Option<u64>>()) as *const Node<T>, self.len() as usize) 
    }
  }
}
impl<T> DerefMut for Mem<T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { 
      std::slice::from_raw_parts_mut(
        self.mmap.as_mut_ptr()
          .add(mem::size_of::<u64>())
          .add(mem::size_of::<Option<u64>>()) as *mut Node<T>, self.len() as usize) 
    }
  }
}
impl<T> AsRef<[Node<T>]> for Mem<T> {
  #[inline]
  fn as_ref(&self) -> &[Node<T>] {
    self.deref()
  }
}
impl<T> AsMut<[Node<T>]> for Mem<T> {
  #[inline]
  fn as_mut(&mut self) -> &mut [Node<T>] {
    self.deref_mut()
  }
}


pub struct RBTree<T> {
  file: File,
  mem: Mem<T>,
  capacity: u64,
}

impl<T: Default + Copy> RBTree<T> {
  const LEAST_CAPACITY: u64 = 4096;

  pub fn create(file: File) -> Result<Self, Error> {
    let len = file.metadata()?.len();
    let capacity = if len < Self::LEAST_CAPACITY {
      file.set_len(Self::LEAST_CAPACITY)?;
      Self::LEAST_CAPACITY
    } else {
      len
    };
    let mem = Mem::new(unsafe{ MmapMut::map_mut(&file)? });
    Ok(Self { file, mem, capacity })
  }

  fn extend(&mut self) -> Result<(), Error> {
    self.capacity *= 2;
    self.file.set_len(self.capacity)?;
    self.mem = Mem::new(unsafe{ MmapMut::map_mut(&self.file)? });
    Ok(())
  }

  fn shrink(&mut self) -> Result<(), Error> {
    self.capacity /= 2;
    self.file.set_len(self.capacity)?;
    self.mem = Mem::new(unsafe{ MmapMut::map_mut(&self.file)? });
    Ok(())
  }

  fn new_node(&mut self) -> Result<u64, Error> {
    if self.mem.occupy() + mem::size_of::<Node<T>>() > self.capacity as usize {
      self.extend()?;
    }
    let n = self.mem.push();
    self.mem[n as usize] = Node::zero();
    Ok(n)
  }

  #[inline]
  fn assign_tree(&mut self, x: u64, y: u64) {
    let parent = self.mem[x as usize].parent;
    self.mem[y as usize].parent = parent;
    if let Some(i) = parent {
      if Some(x) == self.mem[i as usize].left {
        self.mem[i as usize].left = Some(y)
      } else {
        self.mem[i as usize].right = Some(y);
      }
    } else {
      self.mem.set_root(Some(y));
    }
  }

  #[inline]
  fn assign_parent_left(&mut self, x: u64, y: u64) {
    let left = self.mem[x as usize].left;
    self.mem[y as usize].left = left;
    if let Some(i) = left {
      self.mem[i as usize].parent = Some(y);
    }
  }

  #[inline]
  fn assign_parent_right(&mut self, x: u64, y: u64) {
    let right = self.mem[x as usize].right;
    self.mem[y as usize].right = right;
    if let Some(i) = right {
      self.mem[i as usize].parent = Some(y);
    }
  }

  #[inline]
  fn assign_left(&mut self, x: u64, y: Option<u64>) {
    self.mem[x as usize].left = y;
    if let Some(i) = y {
      self.mem[i as usize].parent = Some(x);
    }
  }

  #[inline]
  fn assign_right(&mut self, x: u64, y: Option<u64>) {
    self.mem[x as usize].right = y;
    if let Some(i) = y {
      self.mem[i as usize].parent = Some(x);
    }
  }

  fn del_node(&mut self, x: u64) -> Result<(), Error> {
    let last = self.mem.len()-1;
    self.assign_tree(last, x);
    self.assign_parent_left(last, x);
    self.assign_parent_right(last, x);
    self.mem[x as usize].val = self.mem[last as usize].val;
    self.mem[last as usize] = Node::zero();
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
    let rc = self.mem[x as usize].right.expect("The right node must exist when rotating left.");
    self.assign_right(x, self.mem[rc as usize].left);
    self.assign_tree(x, rc);
    self.assign_left(rc, Some(x));
  }

  fn rotate_right(&mut self, x: u64) {
    let lc = self.mem[x as usize].left.expect("The left node must exist when rotating right.");
    self.assign_left(x, self.mem[lc as usize].right);
    self.assign_tree(x, lc);
    self.assign_right(lc, Some(x));
  }

  /*

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

  */
}

#[cfg(test)]
mod tests {
  use super::*;
  use tempfile::tempfile;

  #[test]
  fn test_create() -> Result<(), Error> {
    let file = tempfile()?;
    let tree: RBTree<u64> = RBTree::create(file)?;
    assert!(tree.capacity >= RBTree::<u64>::LEAST_CAPACITY);
    assert_eq!(tree.mem.len(), 0);
    assert_eq!(tree.mem.root(), None);
    Ok(())
  }

  fn construct_tree(tree: &mut RBTree<u64>) -> Result<(), Error> {
    
    Ok(())
  }

  #[test]
  fn test_new_node() -> Result<(), Error> {
    let file = tempfile()?;
    let mut tree: RBTree<u64> = RBTree::create(file)?;
    for _ in 0..500 {
      tree.new_node()?;
    }
    let occupied = mem::size_of::<u64>() + mem::size_of::<Option<u64>>() + mem::size_of::<Node<u64>>() * 500;
    assert_eq!(tree.mem.occupy(), occupied);
    let cap = (occupied as f64).log2().ceil().exp2() as usize;
    assert_eq!(tree.capacity, cap as u64);
    Ok(())
  }
}