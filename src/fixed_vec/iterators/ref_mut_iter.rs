use std::marker::PhantomData;

use crate::fixed_vec::{FixedVec, owner_tag::DropPolicy};

pub struct FixedVecRefMutIter<'a, T> {
    ptr: *mut T,
    idx: usize,
    item_count: usize,
    _lifetime: PhantomData<&'a T>,
}

impl<'a, T> FixedVecRefMutIter<'a, T> {
    pub fn new(ptr: *mut T, item_count: usize) -> Self {
        Self {
            ptr,
            idx: 0,
            item_count,
            _lifetime: PhantomData,
        }
    }
}

impl<'a, T> Iterator for FixedVecRefMutIter<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.item_count {
            return None;
        }
        self.idx += 1;
        if const { size_of::<T>() == 0 } {
            return unsafe { Some(&mut *std::ptr::NonNull::dangling().as_ptr()) };
        }
        Some(unsafe { &mut *self.ptr.add(self.idx - 1) })
    }
}

impl<'a, T, Policy: DropPolicy<T>> IntoIterator for &'a mut FixedVec<T, Policy> {
    type Item = &'a mut T;

    type IntoIter = FixedVecRefMutIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        FixedVecRefMutIter::new(self.ptr, self.len)
    }
}
