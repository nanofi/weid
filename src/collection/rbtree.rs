use failure::Error;
use memmap::MmapMut;
use std::cmp::{Ord, Ordering};
use std::fs::File;
use std::mem;
use super::Mem;

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

impl<T: Default> Default for Node<T> {
  fn default() -> Self { Self::zero() }
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

#[repr(C)]
struct RBTreeMeta {
  root: Option<u64>,
}

pub struct RBTree<T> {
  file: File,
  mem: Mem<Node<T>, RBTreeMeta>,
  capacity: u64,
}

impl<T: std::fmt::Display> RBTree<T> {
  fn fmt_inner(&self, f: &mut std::fmt::Formatter<'_>, node: u64) -> std::fmt::Result {
    let left = self.mem[node].left;
    let right = self.mem[node].right;
    if left.is_some() || right.is_some() {
      if let Some(l) = left {
        write!(
          f,
          "  {} -> {};\n",
          self.mem[node].val, self.mem[l].val
        )?;
        self.fmt_inner(f, l)?;
      } else {
        write!(
          f,
          "  left{0} [shape=point, label=\"\"];\n  {0} -> left{0};\n",
          self.mem[node].val
        )?;
      }
      if let Some(r) = right {
        write!(
          f,
          "  {} -> {};\n",
          self.mem[node].val, self.mem[r].val
        )?;
        self.fmt_inner(f, r)?;
      } else {
        write!(
          f,
          "  right{0} [shape=point, label=\"\"];\n  {0} -> right{0};\n",
          self.mem[node].val
        )?;
      }
    }
    Ok(())
  }
}
impl<T: std::fmt::Display> std::fmt::Display for RBTree<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "digraph G {{\n  graph [ordering=\"out\"];\n")?;
    for i in 0..self.mem.len() {
      let color = if self.mem[i].is_black() {
        "black"
      } else {
        "red"
      };
      write!(
        f,
        "  {} [label=\"{},{0}\", color=\"{}\"];\n",
        self.mem[i].val, i, color
      )?;
    }
    if let Some(r) = self.mem.meta().root {
      self.fmt_inner(f, r)?;
    }
    write!(f, "}}")?;
    Ok(())
  }
}

impl<T: Default + Ord + Copy> RBTree<T> {
  const LEAST_CAPACITY: u64 = 4096;

  pub fn create(file: File) -> Result<Self, Error> {
    let len = file.metadata()?.len();
    let capacity = if len < Self::LEAST_CAPACITY {
      file.set_len(Self::LEAST_CAPACITY)?;
      Self::LEAST_CAPACITY
    } else {
      len
    };
    let mem = Mem::new(unsafe { MmapMut::map_mut(&file)? });
    Ok(Self {
      file,
      mem,
      capacity,
    })
  }

  fn extend(&mut self) -> Result<(), Error> {
    self.capacity *= 2;
    self.file.set_len(self.capacity)?;
    self.mem = Mem::new(unsafe { MmapMut::map_mut(&self.file)? });
    Ok(())
  }

  fn shrink(&mut self) -> Result<(), Error> {
    self.capacity /= 2;
    self.file.set_len(self.capacity)?;
    self.mem = Mem::new(unsafe { MmapMut::map_mut(&self.file)? });
    Ok(())
  }

  fn new_node(&mut self) -> Result<u64, Error> {
    if self.mem.occupy() + mem::size_of::<Node<T>>() > self.capacity as usize {
      self.extend()?;
    }
    let n = self.mem.push();
    self.mem[n] = Node::zero();
    Ok(n)
  }

  #[inline]
  fn assign_tree(&mut self, x: u64, y: u64) {
    let parent = self.mem[x].parent;
    self.mem[y].parent = parent;
    if let Some(i) = parent {
      if Some(x) == self.mem[i].left {
        self.mem[i].left = Some(y)
      } else {
        self.mem[i].right = Some(y);
      }
    } else {
      self.mem.meta_mut().root = Some(y);
    }
  }

  #[inline]
  fn assign_parent_left(&mut self, x: u64, y: u64) {
    let left = self.mem[x].left;
    self.mem[y].left = left;
    if let Some(i) = left {
      self.mem[i].parent = Some(y);
    }
  }

  #[inline]
  fn assign_parent_right(&mut self, x: u64, y: u64) {
    let right = self.mem[x].right;
    self.mem[y].right = right;
    if let Some(i) = right {
      self.mem[i].parent = Some(y);
    }
  }

  #[inline]
  fn assign_left(&mut self, x: u64, y: Option<u64>) {
    self.mem[x].left = y;
    if let Some(i) = y {
      self.mem[i].parent = Some(x);
    }
  }

  #[inline]
  fn assign_right(&mut self, x: u64, y: Option<u64>) {
    self.mem[x].right = y;
    if let Some(i) = y {
      self.mem[i].parent = Some(x);
    }
  }

  fn del_node(&mut self, x: u64) -> Result<(), Error> {
    let last = self.mem.len() - 1;
    self.assign_tree(last, x);
    self.assign_parent_left(last, x);
    self.assign_parent_right(last, x);
    self.mem[x].val = self.mem[last].val;
    self.mem.pop();
    if self.mem.occupy() < (self.capacity) as usize / 2 {
      self.shrink()?;
    }
    Ok(())
  }

  #[inline]
  fn swap_color(&mut self, x: u64, y: u64) {
    let x = x;
    let y = y;
    let c = self.mem[x].color;
    self.mem[x].color = self.mem[y].color;
    self.mem[y].color = c;
  }

  fn rotate_left(&mut self, x: u64) {
    let rc = self.mem[x]
      .right
      .expect("The right node must exist when rotating left.");
    self.assign_right(x, self.mem[rc].left);
    self.assign_tree(x, rc);
    self.assign_left(rc, Some(x));
  }

  fn rotate_right(&mut self, x: u64) {
    let lc = self.mem[x]
      .left
      .expect("The left node must exist when rotating right.");
    self.assign_left(x, self.mem[lc].right);
    self.assign_tree(x, lc);
    self.assign_right(lc, Some(x));
  }

  fn add_bst(&mut self, val: T) -> Result<u64, Error> {
    let mut x = self.mem.meta().root;
    let mut p = None;
    let mut ord = Ordering::Equal;
    while x.is_some() {
      let i = x.unwrap();
      p = x;
      ord = val.cmp(&self.mem[i].val);
      match ord {
        Ordering::Less => x = self.mem[i].left,
        Ordering::Greater => x = self.mem[i].right,
        _ => bail!("Cannot add the existing item."),
      }
    }
    let node = self.new_node()?;
    self.mem[node].val = val;
    self.mem[node].parent = p;
    self.mem[node].to_red();
    match ord {
      Ordering::Less => self.mem[p.unwrap()].left = Some(node),
      Ordering::Greater => self.mem[p.unwrap()].right = Some(node),
      Ordering::Equal => self.mem.meta_mut().root = Some(node),
    }
    Ok(node)
  }

  fn del_bst(&mut self, x: Option<u64>, val: T) -> Option<u64> {
    let x = match x {
      None => return None,
      Some(x) => x,
    };
    match val.cmp(&self.mem[x].val) {
      Ordering::Less => self.del_bst(self.mem[x].left, val),
      Ordering::Greater => self.del_bst(self.mem[x].right, val),
      Ordering::Equal => {
        if let (Some(_), Some(r)) = (self.mem[x].left, self.mem[x].right) {
          let mut m = r;
          while let Some(n) = self.mem[m].left {
            m = n;
          }
          let v = self.mem[m].val;
          self.mem[x].val = v;
          Some(m)
        } else {
          Some(x)
        }
      }
    }
  }

  #[inline]
  fn is_red<X: Into<Option<u64>>>(&self, x: X) -> bool {
    let x = x.into();
    x.is_some() && self.mem[x.unwrap()].is_red()
  }
  #[inline]
  fn is_black<X: Into<Option<u64>>>(&self, x: X) -> bool {
    let x = x.into();
    x.is_none() || self.mem[x.unwrap()].is_black()
  }

  pub fn new(&mut self) -> Result<T, Error> {
    unimplemented!();
  }

  pub fn add(&mut self, val: T) -> Result<(), Error> {
    let mut x = self.add_bst(val)?;
    while Some(x) != self.mem.meta().root && self.is_red(x) && self.is_red(self.mem[x].parent) {
      let mut p = self.mem[x].parent.unwrap();
      let g = self.mem[p].parent.unwrap();
      if Some(p) == self.mem[g].left {
        let u = self.mem[g].right;
        if self.is_red(u) {
          let u = u.unwrap();
          self.mem[u].to_black();
          self.mem[p].to_black();
          self.mem[g].to_red();
          x = g;
        } else {
          if Some(x) == self.mem[p].right {
            self.rotate_left(p);
            mem::swap(&mut x, &mut p);
          }
          self.rotate_right(g);
          self.swap_color(p, g);
          x = p;
        }
      } else {
        let u = self.mem[g].left;
        if self.is_red(u) {
          let u = u.unwrap();
          self.mem[u].to_black();
          self.mem[p].to_black();
          self.mem[g].to_red();
          x = g;
        } else {
          if Some(x) == self.mem[p].left {
            self.rotate_right(p);
            mem::swap(&mut x, &mut p);
          }
          self.rotate_left(g);
          self.swap_color(p, g);
          x = p;
        }
      }
    }
    let r = self.mem.meta().root.unwrap();
    self.mem[r].to_black();
    Ok(())
  }

  fn del_dblack(&mut self, p: Option<u64>, x: Option<u64>) {
    let p = match p {
      None => return,
      Some(p) => p,
    };
    if x == self.mem[p].left {
      let s = match self.mem[p].right {
        None => return self.del_dblack(self.mem[p].parent, Some(p)),
        Some(s) => s,
      };
      if self.mem[s].is_red() {
        self.rotate_left(p);
        self.mem[s].to_black();
        self.mem[p].to_red();
        self.del_dblack(Some(p), x);
      } else {
        let l = self.mem[s].left;
        let r = self.mem[s].right;
        if self.is_black(l) && self.is_black(r) {
          self.mem[s].to_red();
          if self.mem[p].is_red() {
            self.mem[p].to_black();
          } else {
            self.del_dblack(self.mem[p].parent, Some(p));
          }
        } else {
          if self.is_red(l) {
            self.mem[l.unwrap()].color = self.mem[p].color;
            self.rotate_right(s);
          } else {
            self.mem[r.unwrap()].color = self.mem[s].color;
            self.mem[s].color = self.mem[p].color;
          }
          self.rotate_left(p);
        }
      }
    } else {
      let s = match self.mem[p].left {
        None => return self.del_dblack(self.mem[p].parent, Some(p)),
        Some(s) => s,
      };
      if self.mem[s].is_red() {
        self.rotate_right(p);
        self.mem[s].to_black();
        self.mem[p].to_red();
        self.del_dblack(Some(p), x);
      } else {
        let l = self.mem[s].left;
        let r = self.mem[s].right;
        if self.is_black(l) && self.is_black(r) {
          self.mem[s].to_red();
          if self.mem[p].is_red() {
            self.mem[p].to_black();
          } else {
            self.del_dblack(self.mem[p].parent, Some(p));
          }
        } else {
          if self.is_red(r) {
            self.mem[r.unwrap()].color = self.mem[p].color;
            self.rotate_left(s);
          } else {
            self.mem[l.unwrap()].color = self.mem[s].color;
            self.mem[s].color = self.mem[p].color;
          }
          self.rotate_right(p);
        }
      }
    }
  }

  pub fn del(&mut self, val: T) -> Result<(), Error> {
    let x = self.del_bst(self.mem.meta().root, val);
    let x = match x {
      None => return Ok(()),
      Some(x) if Some(x) == self.mem.meta().root => {
        self.mem.meta_mut().root = None;
        return Ok(());
      },
      Some(x) => x,
    };
    let c = self.mem[x].left.and(self.mem[x].right);
    if self.mem[x].is_red() || (c.is_some() && self.mem[c.unwrap()].is_red()) {
      let p = self.mem[x].parent.unwrap();
      if Some(x) == self.mem[p].left {
        self.mem[p].left = c;
      } else {
        self.mem[p].right = c;
      }
      if let Some(c) = c {
        self.mem[c].to_black();
        self.mem[c].parent = Some(p);
      }
    } else {
      let p = self.mem[x].parent;
      if let Some(p) = p {
        if Some(x) == self.mem[p].left {
          self.mem[p].left = c
        } else {
          self.mem[p].right = c
        }
      } else {
        self.mem.meta_mut().root = c;
      }
      if let Some(c) = c {
        self.mem[c].parent = p;
      }
      self.del_dblack(p, c);
    }
    self.del_node(x)?;
    let r = self.mem.meta().root.unwrap();
    self.mem[r].to_black();
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tempfile::tempfile;
  use rand::Rng;

  fn vals() -> Vec<u64> {
    vec![
      6531, 7872, 6576, 8533, 5085, 2817, 9887, 3796, 1282, 5573, 8589, 3078, 590, 1494, 3295,
      6609, 2587, 5230, 5101, 6358, 2359, 6520, 8487, 9520, 981, 8192, 1044, 25, 3409, 1826, 7563,
      8815, 7790, 4136, 2868, 617, 6433, 3320, 110, 9427, 3556, 1573, 8474, 3794, 4277, 7194, 3708,
      654, 2821, 156, 476, 3343, 387, 3858, 522, 8810, 2947, 8774, 3854, 5693, 9512, 8942, 2646,
      3561, 1760, 67, 3372, 6540, 3447, 8243, 9859, 5944, 7580, 5610, 5478, 1286, 9347, 8831, 8490,
      4875, 465, 9761, 2545, 5496, 6120, 9771, 7852, 9114, 9870, 96, 2068, 8222, 4859, 5872, 505,
      2031, 8440, 6501, 9836, 3554,
    ]
  }
  fn vals_sorted() -> Vec<u64> {
    let mut v = vals();
    v.sort();
    v
  }

  #[test]
  fn test_create() -> Result<(), Error> {
    let file = tempfile()?;
    let tree: RBTree<u64> = RBTree::create(file)?;
    assert!(tree.capacity >= RBTree::<u64>::LEAST_CAPACITY);
    assert_eq!(tree.mem.len(), 0);
    assert_eq!(tree.mem.meta().root, None);
    Ok(())
  }

  fn construct_tree(tree: &mut RBTree<u64>) -> Result<(), Error> {
    for v in vals() {
      tree.add_bst(v)?;
    }
    Ok(())
  }

  #[test]
  fn test_add_del_node() -> Result<(), Error> {
    let file = tempfile()?;
    let mut tree: RBTree<u64> = RBTree::create(file)?;
    for _ in 0..500 {
      tree.new_node()?;
    }
    let occupied =
      mem::size_of::<u64>() + mem::size_of::<Option<u64>>() + mem::size_of::<Node<u64>>() * 500;
    assert_eq!(tree.mem.occupy(), occupied);
    let cap = (occupied as f64).log2().ceil().exp2();
    assert_eq!(tree.capacity, cap as u64);
    for i in 0..490 {
      let t = i % tree.mem.len();
      tree.del_node(t as u64)?;
    }
    let occupied =
      mem::size_of::<u64>() + mem::size_of::<Option<u64>>() + mem::size_of::<Node<u64>>() * 10;
    assert_eq!(tree.mem.occupy(), occupied);
    let cap = (occupied as f64).log2().ceil().exp2();
    assert_eq!(tree.capacity, cap as u64);
    Ok(())
  }

  impl<T: Copy> RBTree<T> {
    fn dfs_inner(&self, b: &mut Vec<T>, node: u64) {
      if let Some(l) = self.mem[node].left {
        self.dfs_inner(b, l);
      }
      b.push(self.mem[node].val);
      if let Some(r) = self.mem[node].right {
        self.dfs_inner(b, r);
      }
    }
    fn dfs(&self) -> Vec<T> {
      let mut b = vec![];
      if let Some(r) = self.mem.meta().root {
        self.dfs_inner(&mut b, r);
      }
      b
    }
  }

  impl<T> RBTree<T> {
    fn assert_root(&self, r: usize) {
      assert_eq!(self.mem.meta().root, Some(r as u64));
      assert_eq!(self.mem[r].parent, None);
    }
    fn assert_left_child<C: Into<Option<u64>>>(&self, p: usize, c: C) {
      let c = c.into();
      assert_eq!(self.mem[p].left, c);
      if let Some(c) = c {
        assert_eq!(self.mem[c].parent, Some(p as u64));
      }
    }
    fn assert_right_child<C: Into<Option<u64>>>(&self, p: usize, c: C) {
      let c = c.into();
      assert_eq!(self.mem[p].right, c);
      if let Some(c) = c {
        assert_eq!(self.mem[c].parent, Some(p as u64));
      }
    }
  }

  #[test]
  fn test_add_bst() -> Result<(), Error> {
    let file = tempfile()?;
    let mut tree: RBTree<u64> = RBTree::create(file)?;
    construct_tree(&mut tree)?;
    let vs = vals_sorted();
    assert_eq!(vs, tree.dfs());
    Ok(())
  }

  #[test]
  fn test_del_bst1() -> Result<(), Error> {
    let file = tempfile()?;
    let mut tree: RBTree<u64> = RBTree::create(file)?;
    construct_tree(&mut tree)?;
    let x = tree.del_bst(tree.mem.meta().root, 6531);
    assert_eq!(x, Some(67));
    assert_eq!(tree.mem[0usize].val, 6540);
    Ok(())
  }
  #[test]
  fn test_del_bst2() -> Result<(), Error> {
    let file = tempfile()?;
    let mut tree: RBTree<u64> = RBTree::create(file)?;
    construct_tree(&mut tree)?;
    let x = tree.del_bst(tree.mem.meta().root, 8533);
    assert_eq!(x, Some(10));
    assert_eq!(tree.mem[3usize].val, 8589);
    Ok(())
  }

  #[test]
  fn test_rotate_left1() -> Result<(), Error> {
    let file = tempfile()?;
    let mut tree: RBTree<u64> = RBTree::create(file)?;
    construct_tree(&mut tree)?;
    tree.rotate_left(19);
    tree.assert_right_child(9, 21);
    tree.assert_left_child(21, 19);
    tree.assert_right_child(19, 36);
    Ok(())
  }
  #[test]
  fn test_rotate_left2() -> Result<(), Error> {
    let file = tempfile()?;
    let mut tree: RBTree<u64> = RBTree::create(file)?;
    construct_tree(&mut tree)?;
    tree.rotate_left(36);
    tree.assert_left_child(21, 97);
    tree.assert_left_child(97, 36);
    tree.assert_right_child(36, None);
    Ok(())
  }
  #[test]
  fn test_rotate_left3() -> Result<(), Error> {
    let file = tempfile()?;
    let mut tree: RBTree<u64> = RBTree::create(file)?;
    construct_tree(&mut tree)?;
    tree.rotate_left(0);
    tree.assert_root(1);
    tree.assert_left_child(1, 0);
    tree.assert_right_child(0, 2);
    Ok(())
  }

  #[test]
  fn test_rotate_right1() -> Result<(), Error> {
    let file = tempfile()?;
    let mut tree: RBTree<u64> = RBTree::create(file)?;
    construct_tree(&mut tree)?;
    tree.rotate_right(9);
    tree.assert_right_child(4, 17);
    tree.assert_right_child(17, 9);
    tree.assert_left_child(9, 74);
    Ok(())
  }
  #[test]
  fn test_rotate_right2() -> Result<(), Error> {
    let file = tempfile()?;
    let mut tree: RBTree<u64> = RBTree::create(file)?;
    construct_tree(&mut tree)?;
    tree.rotate_right(46);
    tree.assert_left_child(43, 63);
    tree.assert_right_child(63, 46);
    tree.assert_left_child(46, None);
    Ok(())
  }
  #[test]
  fn test_rotate_right3() -> Result<(), Error> {
    let file = tempfile()?;
    let mut tree: RBTree<u64> = RBTree::create(file)?;
    construct_tree(&mut tree)?;
    tree.rotate_right(0);
    tree.assert_root(4);
    tree.assert_right_child(4, 0);
    tree.assert_left_child(0, 9);
    Ok(())
  }


  struct BlackHeightAsserter<'a, T> {
    tree: &'a RBTree<T>,
    height: Option<usize>,
  }
  impl<T: Default + Ord + Copy> RBTree<T> {
    fn assert_constraint(&self) {
      assert!(self.is_black(self.mem.meta().root));
      self.assert_black_height();
    }

    fn assert_black_height(&self) {
      BlackHeightAsserter { tree: self, height: None }.assert();
    }
  }
  impl<'a, T> BlackHeightAsserter<'a, T> {
    fn assert_recur(&mut self, x: Option<u64>, h: usize) {
      if let Some(x) = x {
        let h = if self.tree.mem[x].is_black() { h + 1 } else { h };
        self.assert_recur(self.tree.mem[x].left, h);
        self.assert_recur(self.tree.mem[x].right, h);
      } else {
        let h = h + 1;
        if let Some(e) = self.height {
          assert_eq!(h, e);
        } else {
          self.height = h.into();
        }
      }
    }

    fn assert(&mut self) {
      self.assert_recur(self.tree.mem.meta().root, 0);
    }
  }

  #[test]
  fn test_byte_size() {
    assert_eq!(mem::size_of::<Node<u64>>(), 64);
    assert_eq!(mem::size_of::<Option<u64>>(), 16);
  }

  #[test]
  fn test_add() -> Result<(), Error> {
    let file = tempfile()?;
    let mut tree: RBTree<u64> = RBTree::create(file)?;
    for v in vals() {
      tree.add(v)?;
    }
    tree.assert_constraint();

    Ok(())
  }

  #[test]
  fn test_random_add() -> Result<(), Error> {
    let file = tempfile()?;
    let mut tree: RBTree<u64> = RBTree::create(file)?;
    let mut rng = rand::thread_rng();
    for _ in 0..10000 {
      tree.add(rng.gen())?;
    }
    tree.assert_constraint();
    
    Ok(())
  }

  #[test]
  fn test_del() -> Result<(), Error> {
    let file = tempfile()?;
    let mut tree: RBTree<u64> = RBTree::create(file)?;
    for v in vals() {
      tree.add(v)?;
    }

    let mut c = 0;
    for v in vals() {
      c += 1;
      tree.del(v)?;
      if c >= vals().len()/2 {
        break;
      }
    }

    tree.assert_constraint();

    Ok(())
  }
}
