use core::slice;
use std::{
    alloc::{Layout, alloc, dealloc, realloc},
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    ops::Deref,
    ptr,
};

use crate::fixed_vec::owner_tag::{DropPolicy, Owned, Reference};

pub mod owner_tag;

pub mod iterators;

pub struct FixedVec<T, Policy: DropPolicy<T> = Owned> {
    ptr: *mut T,
    capacity: usize,
    len: usize,
    _drop_policy: PhantomData<Policy>,
}

impl<'a, T> FixedVec<T, Reference<'a>> {
    pub unsafe fn new_from_parts(ptr: *mut T, capacity: usize, len: usize) -> Self {
        Self {
            ptr,
            capacity,
            len,
            _drop_policy: PhantomData,
        }
    }

    pub fn new_from_slice(r: &'a mut [MaybeUninit<T>], len: usize) -> Self {
        let ptr = r.as_ptr() as *mut T;
        Self {
            ptr,
            capacity: len,
            len,
            _drop_policy: PhantomData,
        }
    }
}

impl<T> FixedVec<T, Owned> {
    pub fn new(capacity: usize) -> Self {
        let layout = Layout::array::<T>(capacity).expect("Capacity too large");
        let ptr = unsafe { alloc(layout) as *mut T };

        Self {
            ptr,
            capacity,
            len: 0,
            _drop_policy: PhantomData,
        }
    }
}

impl<T, Policy: DropPolicy<T>> Drop for FixedVec<T, Policy> {
    fn drop(&mut self) {
        Policy::drop_fixed_vec(self);
    }
}

impl<T, Policy: DropPolicy<T>> FixedVec<T, Policy> {
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
            return Some(item);
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
        unsafe { Some(self.ptr.add(self.len).read()) }
    }
    pub fn get(&self, idx: usize) -> Option<&T> {
        if self.len >= idx {
            return None;
        }
        Some(unsafe { &*self.ptr.add(idx) })
    }
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if self.len >= idx {
            return None;
        }
        Some(unsafe { &mut *self.ptr.add(idx) })
    }
    pub fn to_boxed_slice(self) -> Box<[T]> {
        if const { size_of::<T>() == 0 } {
            return unsafe { Box::from_raw(ptr::slice_from_raw_parts_mut(ptr::dangling_mut(), 0)) };
        }
        let layout = Layout::array::<T>(self.capacity()).expect("Capacity too high");
        if self.len() == 0 {
            unsafe {
                dealloc(self.ptr as *mut u8, layout);
            }
            return unsafe { Box::from_raw(ptr::slice_from_raw_parts_mut(ptr::dangling_mut(), 0)) };
        }
        let new_ptr = unsafe { realloc(self.ptr as *mut u8, layout, self.len()) as *mut T };
        if new_ptr.is_null() {
            std::alloc::handle_alloc_error(layout)
        }
        let slice = ptr::slice_from_raw_parts_mut(new_ptr, self.len());
        unsafe { Box::from_raw(slice) }
    }

    pub fn to_vec(self) -> Vec<T> {
        let disabled_fixed_vec = ManuallyDrop::new(self);
        let (ptr, len, capacity) = (
            disabled_fixed_vec.ptr,
            disabled_fixed_vec.len,
            disabled_fixed_vec.capacity,
        );

        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    }
}

impl<T, Policy: DropPolicy<T>> Deref for FixedVec<T, Policy> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { &slice::from_raw_parts_mut(self.ptr, self.len)[..] }
    }
}

impl<T> From<Box<[T]>> for FixedVec<T, Owned> {
    fn from(value: Box<[T]>) -> Self {
        let disabled_box = ManuallyDrop::new(value);
        let ptr = disabled_box.as_ptr() as *mut T;
        let len = disabled_box.len();

        Self {
            ptr,
            capacity: len,
            len,
            _drop_policy: PhantomData,
        }
    }
}
impl<T> From<Box<[MaybeUninit<T>]>> for FixedVec<T, Owned> {
    fn from(value: Box<[MaybeUninit<T>]>) -> Self {
        let disabled_box = ManuallyDrop::new(value);
        let ptr = disabled_box.as_ptr() as *mut T;
        let len = disabled_box.len();

        Self {
            ptr,
            capacity: len,
            len: 0,
            _drop_policy: PhantomData,
        }
    }
}

impl<T> From<(Box<[MaybeUninit<T>]>, usize)> for FixedVec<T, Owned> {
    fn from(value: (Box<[MaybeUninit<T>]>, usize)) -> Self {
        let disabled_box = ManuallyDrop::new(value.0);
        let ptr = disabled_box.as_ptr() as *mut T;
        let len = disabled_box.len();

        Self {
            ptr,
            capacity: len,
            len: value.1,
            _drop_policy: PhantomData,
        }
    }
}

impl<T> From<Vec<T>> for FixedVec<T, Owned> {
    fn from(value: Vec<T>) -> Self {
        let disabled_vec = ManuallyDrop::new(value);
        let ptr = disabled_vec.as_ptr() as *mut T;
        let capacity = disabled_vec.capacity();
        let len = disabled_vec.len();

        Self {
            ptr,
            capacity,
            len,
            _drop_policy: PhantomData,
        }
    }
}
