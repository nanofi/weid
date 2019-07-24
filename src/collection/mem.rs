
use memmap::MmapMut;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::mem;
use std::slice::SliceIndex;
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

pub struct Mem<T, M = ()> {
  mmap: MmapMut,
  pt: PhantomData<M>,
  pm: PhantomData<T>,
}

impl<T, M> Mem<T, M> {
  pub fn new(mmap: MmapMut) -> Self {
    Self {
      mmap,
      pt: PhantomData,
      pm: PhantomData,
    }
  }

  #[inline]
  pub fn len(&self) -> u64 {
    unsafe { *(self.mmap.as_ptr() as *const u64) }
  }
  #[inline]
  fn len_mut(&mut self) -> &mut u64 {
    unsafe { &mut *(self.mmap.as_mut_ptr() as *mut u64) }
  }
  #[inline]
  pub fn meta(&self) -> &M {
    unsafe { &*(self.mmap.as_ptr().add(mem::size_of::<u64>()) as *const M) }
  }
  #[inline]
  pub fn meta_mut(&mut self) -> &mut M {
    unsafe { &mut *(self.mmap.as_mut_ptr().add(mem::size_of::<u64>()) as *mut M) }
  }

  #[inline]
  pub fn occupy(&self) -> usize {
    mem::size_of::<u64>()
      + mem::size_of::<Option<u64>>()
      + mem::size_of::<T>() * self.len() as usize
  }

  #[inline]
  pub fn push(&mut self) -> u64 {
    let len = self.len();
    *self.len_mut() += 1;
    len
  }
}

impl<T: Default, M> Mem<T, M> {
  #[inline]
  pub fn pop(&mut self) {
    let last = self.len() - 1;
    self[last] = Default::default();
    *self.len_mut() -= 1;
  }
}

impl<T, M> Deref for Mem<T, M> {
  type Target = [T];
  #[inline]
  fn deref(&self) -> &Self::Target {
    unsafe {
      std::slice::from_raw_parts(
        self
          .mmap
          .as_ptr()
          .add(mem::size_of::<u64>())
          .add(mem::size_of::<Option<u64>>()) as *const T,
        self.len() as usize,
      )
    }
  }
}
impl<T, M> DerefMut for Mem<T, M> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe {
      std::slice::from_raw_parts_mut(
        self
          .mmap
          .as_mut_ptr()
          .add(mem::size_of::<u64>())
          .add(mem::size_of::<Option<u64>>()) as *mut T,
        self.len() as usize,
      )
    }
  }
}
impl<T, M> AsRef<[T]> for Mem<T, M> {
  #[inline]
  fn as_ref(&self) -> &[T] {
    self.deref()
  }
}
impl<T, M> AsMut<[T]> for Mem<T, M> {
  #[inline]
  fn as_mut(&mut self) -> &mut [T] {
    self.deref_mut()
  }
}

pub trait MemIndex<T> {
  type Index: SliceIndex<[T]>;
  fn into_index(self) -> Self::Index;
}
impl<T> MemIndex<T> for usize {
  type Index = usize;
  #[inline]
  fn into_index(self) -> Self::Index { self }
}
impl<T> MemIndex<T> for Range<usize> {
  type Index = Range<usize>;
  #[inline]
  fn into_index(self) -> Self::Index { self }
}
impl<T> MemIndex<T> for RangeFrom<usize> {
  type Index = RangeFrom<usize>;
  #[inline]
  fn into_index(self) -> Self::Index { self }
}
impl<T> MemIndex<T> for RangeInclusive<usize> {
  type Index = RangeInclusive<usize>;
  #[inline]
  fn into_index(self) -> Self::Index { self }
}
impl<T> MemIndex<T> for RangeTo<usize> {
  type Index = RangeTo<usize>;
  #[inline]
  fn into_index(self) -> Self::Index { self }
}
impl<T> MemIndex<T> for RangeToInclusive<usize> {
  type Index = RangeToInclusive<usize>;
  #[inline]
  fn into_index(self) -> Self::Index { self }
}
impl<T> MemIndex<T> for u64 {
  type Index = usize;
  #[inline]
  fn into_index(self) -> Self::Index { self as usize }
}
impl<T> MemIndex<T> for Range<u64> {
  type Index = Range<usize>;
  #[inline]
  fn into_index(self) -> Self::Index { Self::Index{ start: self.start as usize, end: self.end as usize } }
}
impl<T> MemIndex<T> for RangeFrom<u64> {
  type Index = RangeFrom<usize>;
  #[inline]
  fn into_index(self) -> Self::Index { Self::Index{ start: self.start as usize } }
}
impl<T> MemIndex<T> for RangeInclusive<u64> {
  type Index = RangeInclusive<usize>;
  #[inline]
  fn into_index(self) -> Self::Index { let (start, end) = self.into_inner(); Self::Index::new(start as usize, end as usize) }
}
impl<T> MemIndex<T> for RangeTo<u64> {
  type Index = RangeTo<usize>;
  #[inline]
  fn into_index(self) -> Self::Index { Self::Index{ end: self.end as usize } }
}
impl<T> MemIndex<T> for RangeToInclusive<u64> {
  type Index = RangeToInclusive<usize>;
  #[inline]
  fn into_index(self) -> Self::Index { Self::Index{ end: self.end as usize } }
}
impl<T> MemIndex<T> for RangeFull {
  type Index = RangeFull;
  #[inline]
  fn into_index(self) -> Self::Index { self }
}


impl<T, M, I: MemIndex<T>> Index<I> for Mem<T, M> {
  type Output = <I::Index as SliceIndex<[T]>>::Output;
  #[inline]
  fn index(&self, i: I) -> &Self::Output {
    &self.deref()[i.into_index()]
  }
}
impl<T, M, I: MemIndex<T>> IndexMut<I> for Mem<T, M> {
  fn index_mut(&mut self, i: I) -> &mut Self::Output {
    &mut self.deref_mut()[i.into_index()]
  }
}