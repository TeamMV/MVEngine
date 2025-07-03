pub mod args;
pub mod mapto;
pub mod savers;

use std::ops::{Deref, DerefMut};

///CAUTION!!! UNSAFE
pub struct CloneMut<'a, T> {
    inner: &'a mut T,
}

impl<'a, T> CloneMut<'a, T> {
    pub fn new(t: &'a mut T) -> Self {
        Self { inner: t }
    }
}

impl<'a, T> Clone for CloneMut<'a, T> {
    fn clone(&self) -> Self {
        let cast = mvutils::unsafe_cast_mut!(self, Self);
        Self { inner: cast.inner }
    }
}

impl<'a, T> Deref for CloneMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, T> DerefMut for CloneMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
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
    unsafe {
        (p as *const R).as_ref().unwrap()
    }
}

pub fn pointee_mut<'a, R>(p: usize) -> &'a mut R {
    unsafe {
        (p as *mut R).as_mut().unwrap()
    }
}

pub fn pointer<T>(t: &T) -> usize {
    unsafe {
        (t as *const T as usize)
    }
}