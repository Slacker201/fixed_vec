use std::marker::PhantomData;

use crate::fixed_vec::{FixedVec, owner_tag::DropPolicy};

pub struct FixedVecRefIter<'a, T> {
    ptr: *const T,
    idx: usize,
    item_count: usize,
    _lifetime: PhantomData<&'a T>,
}

impl<'a, T> FixedVecRefIter<'a, T> {
    pub fn new(ptr: *const T, item_count: usize) -> Self {
        Self {
            ptr,
            idx: 0,
            item_count,
            _lifetime: PhantomData,
        }
    }
}

impl<'a, T: 'a> Iterator for FixedVecRefIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.item_count {
            return None;
        }
        self.idx += 1;
        if const { size_of::<T>() == 0 } {
            return unsafe { Some(&*std::ptr::NonNull::dangling().as_ptr()) };
        }
        Some(unsafe { &*self.ptr.add(self.idx - 1) })
    }
}

impl<'a, T: 'a, Policy: DropPolicy<T>> IntoIterator for &'a FixedVec<T, Policy> {
    type Item = &'a T;

    type IntoIter = FixedVecRefIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        FixedVecRefIter::new(self.ptr(), self.len())
    }
}
