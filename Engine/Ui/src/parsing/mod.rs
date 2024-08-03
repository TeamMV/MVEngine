use parking_lot::RwLock;
use std::str::Bytes;
use std::sync::Arc;

pub mod xml;

pub trait Parser {
    fn next(&self) -> Option<Tag>;

    fn inner(&self) -> Option<Arc<RwLock<dyn Parser>>>
    where
        Self: Sized;

    fn parse(bytes: BytesIter) -> Self
    where
        Self: Sized;
}

pub struct Tag {
    namespace: String,
    name: String,
}

pub struct BytesIter {
    iter: Box<dyn Iterator<Item = u8>>,
}

impl BytesIter {
    pub fn from_iter(iter: impl Iterator<Item = u8> + 'static) -> Self {
        Self {
            iter: Box::new(iter),
        }
    }
}

impl Iterator for BytesIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
