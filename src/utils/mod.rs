pub mod args;
pub mod mapto;
pub mod savers;

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// This type is just an f64 which is expected to be between 0 and 1. Mostly used for percentages.
pub type F0To1 = f64;

pub fn noop<T>(_: T) {}

/// CAUTION!!! UNSAFE
pub struct CopyMut<'a, T> {
    inner: *mut T,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, T> CopyMut<'a, T> {
    pub fn new(t: &'a mut T) -> Self {
        Self {
            inner: t as *mut _,
            _phantom: PhantomData::default()
        }
    }
}

impl<'a, T> From<&'a mut T> for CopyMut<'a, T> {
    fn from(value: &'a mut T) -> Self {
        Self::new(value)
    }
}

impl<'a, T> Clone for CopyMut<'a, T> {
    fn clone(&self) -> Self {
        let cast = mvutils::unsafe_cast_mut!(self, Self);
        Self {
            inner: cast.inner,
            _phantom: PhantomData::default(),
        }
    }
}

impl<T> Copy for CopyMut<'_, T> {}

impl<'a, T: 'a> Deref for CopyMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &'a Self::Target {
        unsafe { &*self.inner }
    }
}

impl<'a, T: 'a> DerefMut for CopyMut<'a, T> {
    fn deref_mut(&mut self) -> &'a mut Self::Target {
        unsafe { &mut *self.inner }
    }
}

pub trait Expect2<T> {
    fn expect2(self, msg: &str) -> T;
}

impl<T, E> Expect2<T> for Result<T, E> {
    fn expect2(self, msg: &str) -> T {
        if let Ok(t) = self {
            return t;
        }
        panic!("{msg}");
    }
}

pub fn pointee<'a, R>(p: usize) -> &'a R {
    unsafe { (p as *const R).as_ref().unwrap() }
}

pub fn pointee_mut<'a, R>(p: usize) -> &'a mut R {
    unsafe { (p as *mut R).as_mut().unwrap() }
}

pub fn pointer<T>(t: &T) -> usize {
    unsafe { (t as *const T as usize) }
}