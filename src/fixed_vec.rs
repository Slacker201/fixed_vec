use std::{alloc::{Layout, alloc}, marker::PhantomData, mem::MaybeUninit};

use crate::fixed_vec::owner_tag::{MemoryPolicy, Owned, Reference};

pub mod owner_tag;


pub(crate) mod iterators;


pub struct FixedVec<T, D: MemoryPolicy<T> = Owned> {
    ptr: *mut T,
    capacity: usize,
    len: usize,
    _drop_policy: PhantomData<D>
}

impl<'a, T> FixedVec<T, Reference<'a>> {
    pub unsafe fn new_from_parts(ptr: *mut T, capacity: usize, len: usize) -> Self {
        Self { ptr, capacity, len, _drop_policy: PhantomData }
    }

    pub fn new_from_slice(r: &'a mut [MaybeUninit<T>], len: usize) -> Self {
        let ptr = r.as_ptr() as *mut T;
        Self { ptr, capacity: len, len, _drop_policy: PhantomData }
    }
}

impl<T> FixedVec<T, Owned> {
    pub fn new(capacity: usize) -> Self {
        let layout = Layout::array::<T>(capacity).expect("Capacity too large");
        let ptr = unsafe {
            alloc(layout) as *mut T
        };

        Self { ptr, capacity, len: 0, _drop_policy: PhantomData }
    }
}


impl<T, D: MemoryPolicy<T>> Drop for FixedVec<T, D> {
    fn drop(&mut self) {
        D::drop_fixed_vec(self);
    }
}

impl<T, D: MemoryPolicy<T>> FixedVec<T, D> {
    pub fn ptr(&self) -> *const T {
        self.ptr as *const T
    }
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn push(&mut self, item: T) -> Option<T> {
        if self.len >= self.capacity {
            return Some(item)
        }
        unsafe {
            self.ptr.add(self.len).write(item);
        }
        self.len += 1;
        None
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        unsafe {
            Some(self.ptr.add(self.len).read())
        }
    }
    pub fn get(&self, idx: usize) -> Option<&T> {
        if self.len >= idx {
            return None;
        }
        Some(
            unsafe {
                &*self.ptr.add(idx)
            }
        )
    }
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if self.len >= idx {
            return None;
        }
        Some(
            unsafe {
                &mut *self.ptr.add(idx)
            }
        )
    }
}