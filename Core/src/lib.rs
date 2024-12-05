#![feature(portable_simd)]
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod asset;
pub mod err;
pub mod input;
pub mod math;
pub mod render;

pub trait OptionGetMapOr<T> {
    fn get_map_or<R>(&self, mapper: fn(&T) -> R, def: R) -> R;
    fn get_mut_map_or<R>(&mut self, mapper: fn(&mut T) -> R, def: R) -> R;
}

impl<T> OptionGetMapOr<T> for Option<T> {
    fn get_map_or<R>(&self, mapper: fn(&T) -> R, def: R) -> R {
        if self.is_some() {
            return mapper(self.as_ref().unwrap());
        }
        def
    }

    fn get_mut_map_or<R>(&mut self, mapper: fn(&mut T) -> R, def: R) -> R {
        if self.is_some() {
            return mapper(self.as_mut().unwrap());
        }
        def
    }
}