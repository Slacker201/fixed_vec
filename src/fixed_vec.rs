use core::slice;
use std::{
    alloc::{Layout, alloc, dealloc, realloc},
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut},
    ptr,
};

use crate::fixed_vec::owner_tag::{DropPolicy, Owned, Reference};

pub mod owner_tag;

pub mod iterators;
/// The fixed vector struct
/// 
/// # Examples
/// 
/// ```rust
/// use fixed_vec::FixedVec;
/// 
/// let mut fv = FixedVec::new(2);
/// assert_eq!(None, fv.push(1));
/// assert_eq!(None, fv.push(2));
/// 
/// assert_eq!(Some(2), fv.pop());
/// assert_eq!(Some(1), fv.pop());
/// ```
pub struct FixedVec<T, Policy: DropPolicy<T> = Owned> {
    ptr: *mut T,
    capacity: usize,
    len: usize,
    _drop_policy: PhantomData<Policy>,
}

impl<'a, T> FixedVec<T, Reference<'a>> {
    /// Creates a new `FixedVec` from raw parts, with the Reference drop policy
    /// 
    /// # Safety
    /// This is unsafe if you do not follow rust's pointer aliasing rules
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use fixed_vec::FixedVec;
    /// let layout = std::alloc::Layout::array::<i32>(2).unwrap();
    /// let ptr = unsafe { std::alloc::alloc(layout) as *mut i32 };
    /// let capacity = 02;
    /// let len = 0;
    /// 
    /// let mut fv = unsafe { FixedVec::new_from_parts(ptr, capacity, len) };
    /// assert_eq!(None, fv.push(1));
    /// assert_eq!(None, fv.push(2));
    /// assert_eq!(fv.get(0), Some(&1));
    /// assert_eq!(fv.get(1), Some(&2));
    /// ```
    pub unsafe fn new_from_parts(ptr: *mut T, capacity: usize, len: usize) -> Self {
        Self {
            ptr,
            capacity,
            len,
            _drop_policy: PhantomData,
        }
    }

    /// Creates a new `FixedVec` using a mutable reference to a slice
    /// 
    /// # Safety
    /// This is unsafe if you pass the incorrect length
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use fixed_vec::FixedVec;
    /// use std::mem::MaybeUninit;
    /// let mut slice = [MaybeUninit::new(1), MaybeUninit::new(2), MaybeUninit::uninit(), MaybeUninit::uninit()];
    /// 
    /// let fv = FixedVec::new_from_slice(&mut slice, 2);
    /// assert_eq!(fv.get(0), Some(&1));
    /// ```
    pub unsafe fn new_from_slice(r: &'a mut [MaybeUninit<T>], len: usize) -> Self {
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
    /// Creates a new FixedVec with a specified capacity and zero items
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use fixed_vec::FixedVec;
    /// 
    /// let mut fv = FixedVec::new(4);
    /// 
    /// assert_eq!(4, fv.capacity());
    /// assert_eq!(0, fv.len());
    /// 
    /// assert_eq!(None, fv.push(0));
    /// 
    /// assert_eq!(fv.get(0), Some(&0));
    /// ```
    pub fn new(capacity: usize) -> Self {
        let layout = Layout::array::<T>(capacity).expect("Capacity too large");
        let ptr = unsafe { alloc(layout) as *mut T };
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout)
        }
        Self {
            ptr,
            capacity,
            len: 0,
            _drop_policy: PhantomData,
        }
    }

    /// Converts the `FixedVec` into a boxed slice
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use fixed_vec::FixedVec;
    /// 
    /// let mut fv = FixedVec::new(4);
    /// 
    /// assert_eq!(None, fv.push(1));
    /// 
    /// let b = fv.to_boxed_slice();
    /// 
    /// assert_eq!(b.len(), 1);
    /// assert_eq!(b.get(0), Some(&1));
    /// assert_eq!(b.get(1), None);
    /// ```
    pub fn to_boxed_slice(self) -> Box<[T]> {
        let disabled_self = ManuallyDrop::new(self);
        if const { size_of::<T>() == 0 } {
            return unsafe { Box::from_raw(ptr::slice_from_raw_parts_mut(ptr::dangling_mut(), 0)) };
        }
        let layout = Layout::array::<T>(disabled_self.capacity()).expect("Capacity too high");
        if disabled_self.len() == 0 {
            unsafe {
                dealloc(disabled_self.ptr as *mut u8, layout);
            }
            return unsafe { Box::from_raw(ptr::slice_from_raw_parts_mut(ptr::dangling_mut(), 0)) };
        }
        let new_ptr = unsafe { realloc(disabled_self.ptr as *mut u8, layout, disabled_self.len()) as *mut T };
        if new_ptr.is_null() {
            std::alloc::handle_alloc_error(layout)
        }
        let slice = ptr::slice_from_raw_parts_mut(new_ptr, disabled_self.len());
        unsafe { Box::from_raw(slice) }
    }

    /// Converts the `FixedVec` into a vector
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use fixed_vec::FixedVec;
    /// 
    /// let mut fv = FixedVec::new(4);
    /// 
    /// assert_eq!(None, fv.push(1));
    /// 
    /// let v = fv.to_vec();
    /// 
    /// assert_eq!(v.capacity(), 4);
    /// assert_eq!(v.len(), 1);
    /// assert_eq!(v.get(0), Some(&1));
    /// assert_eq!(v.get(1), None);
    /// ```
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

impl<T, Policy: DropPolicy<T>> Drop for FixedVec<T, Policy> {
    fn drop(&mut self) {
        Policy::drop_fixed_vec(self);
    }
}

impl<T, Policy: DropPolicy<T>> FixedVec<T, Policy> {
    /// Returns the pointer
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use fixed_vec::FixedVec;
    /// 
    /// let fv = FixedVec::new(4);
    /// 
    /// let _ptr = fv.ptr();
    /// ```
    pub fn ptr(&self) -> *const T {
        self.ptr as *const T
    }
    /// Returns the capacity
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use fixed_vec::FixedVec;
    /// let fv = FixedVec::<i32>::new(4);
    /// 
    /// assert_eq!(fv.capacity(), 4);
    /// ```
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    /// Returns the length
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use fixed_vec::FixedVec;
    /// let mut fv = FixedVec::new(4);
    /// 
    /// assert_eq!(fv.len(), 0);
    /// assert_eq!(fv.push(1), None);
    /// assert_eq!(fv.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }
    /// Pushes a value into the vector. 
    /// 
    /// # Returns
    /// Returns the item if there is no more space
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use fixed_vec::FixedVec;
    /// let mut fv = FixedVec::new(2);
    /// 
    /// assert_eq!(fv.push(1), None);
    /// assert_eq!(fv.push(2), None);
    /// assert_eq!(fv.push(3), Some(3));
    /// ```
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
    /// Pops a value from the vector
    /// 
    /// # Returns
    /// Returns `None` if there is no item left
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use fixed_vec::FixedVec;
    /// let mut fv = FixedVec::new(2);
    /// 
    /// assert_eq!(fv.push(1), None);
    /// assert_eq!(fv.push(2), None);
    /// assert_eq!(fv.pop(), Some(2));
    /// assert_eq!(fv.pop(), Some(1));
    /// assert_eq!(fv.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        unsafe { Some(self.ptr.add(self.len).read()) }
    }
    /// Returns a reference to the value at idx, if available
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use fixed_vec::FixedVec;
    /// 
    /// let fv: FixedVec<_> = vec![1, 2, 3].into();
    /// 
    /// assert_eq!(fv.get(0), Some(&1));
    /// assert_eq!(fv.get(1), Some(&2));
    /// assert_eq!(fv.get(2), Some(&3));
    /// 
    /// assert_eq!(fv.get(69), None);
    /// ```
    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx >= self.len {
            return None;
        }
        Some(unsafe { &*self.ptr.add(idx) })
    }
    /// Returns a mutable reference to the value at idx, if available
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use fixed_vec::FixedVec;
    /// 
    /// let mut fv: FixedVec<_> = vec![1, 2, 3].into();
    /// 
    /// assert_eq!(fv.get_mut(0), Some(&mut 1));
    /// assert_eq!(fv.get_mut(1), Some(&mut 2));
    /// assert_eq!(fv.get_mut(2), Some(&mut 3));
    /// 
    /// assert_eq!(fv.get_mut(69), None);
    /// 
    /// *fv.get_mut(0).unwrap() += 5;
    /// 
    /// assert_eq!(fv.get(0), Some(&6));
    /// ```
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx >= self.len {
            return None;
        }
        Some(unsafe { &mut *self.ptr.add(idx) })
    }
}

impl<T, Policy: DropPolicy<T>> Deref for FixedVec<T, Policy> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { &slice::from_raw_parts(self.ptr, self.len)[..] }
    }
}

impl<T, Policy: DropPolicy<T>> DerefMut for FixedVec<T, Policy> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut slice::from_raw_parts_mut(self.ptr, self.len)[..] }
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
