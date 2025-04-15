use std::ops::{Deref, DerefMut};
use mvutils::unsafe_utils::Unsafe;

///CAUTION!!! UNSAFE
pub struct CloneMut<'a, T> {
    inner: &'a mut T
}

impl<'a, T> CloneMut<'a, T> {
    pub fn new(t: &'a mut T) -> Self {
        Self {
            inner: t,
        }
    }
}

impl<'a, T> Clone for CloneMut<'a, T> {
    fn clone(&self) -> Self {
        let cast = mvutils::unsafe_cast_mut!(self, Self);
        Self {
            inner: cast.inner,
        }
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