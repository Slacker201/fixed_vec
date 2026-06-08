use std::{alloc::{Layout, dealloc}, marker::PhantomData, ptr::drop_in_place};

use crate::fixed_vec::FixedVec;

pub trait DropPolicy<T>: Sized {
    fn drop_fixed_vec(fixed_vec: &mut FixedVec<T, Self>);
}

pub struct Owned;
impl<T> DropPolicy<T> for Owned {
    fn drop_fixed_vec(fixed_vec: &mut FixedVec<T, Self>) {
        let ptr = fixed_vec.ptr() as *mut T;
        let len = fixed_vec.len();
        for i in 0..len {
            unsafe { drop_in_place(ptr.add(i)) };
        }
        if const{ size_of::<T>() > 0} {
            let layout = Layout::array::<T>(fixed_vec.capacity()).expect("capacity too big");
            unsafe { dealloc(ptr as *mut u8, layout) };
        }
    }
}
pub struct Reference<'a>(PhantomData<&'a ()>);
impl<'a, T> DropPolicy<T> for Reference<'a> {
    fn drop_fixed_vec(_fixed_vec: &mut FixedVec<T, Self>) {
        // Do nothing because we do not own it
    }
}