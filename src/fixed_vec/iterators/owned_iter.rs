use std::{alloc::{Layout, dealloc}, mem::ManuallyDrop, ptr::drop_in_place};

use crate::fixed_vec::{FixedVec, owner_tag::Owned};

pub struct FixedVecOwnedIter<T> {
    ptr: *mut T,
    idx: usize,
    item_count: usize,
    capacity: usize,
}

impl<T> FixedVecOwnedIter<T> {
    pub fn new(ptr: *mut T, len: usize, capacity: usize) -> Self {
        Self { ptr, idx: 0, item_count: len, capacity }
    }
}

impl<T> Iterator for FixedVecOwnedIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.item_count {
            return None;
        }
        self.idx += 1;
        if const { size_of::<T>() == 0 } {
            return Some(unsafe { std::mem::zeroed() })
        }

        Some(unsafe { self.ptr.add(self.idx-1).read() })
    }
}

impl<T> Drop for FixedVecOwnedIter<T> {
    fn drop(&mut self) {
        for offset in self.idx..self.item_count {
            unsafe {
                drop_in_place(self.ptr.add(offset))
            };
        }
        if const { size_of::<T>() > 0 } {
            let layout = Layout::array::<T>(self.capacity).expect("Capacity too large");
            unsafe {
                dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

impl<T> IntoIterator for FixedVec<T, Owned> {
    type Item = T;

    type IntoIter = FixedVecOwnedIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let disabled_vec = ManuallyDrop::new(self);
        FixedVecOwnedIter::new(disabled_vec.ptr, disabled_vec.len, disabled_vec.capacity)
    }
}