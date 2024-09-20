#![feature(portable_simd)]
#![allow(dead_code)]
#![allow(unused_variables)]

use mvutils::unsafe_utils::DangerousCell;
use std::sync::Arc;

pub mod asset;
pub mod color;
pub mod err;
pub mod input;
pub mod math;
pub mod render;

pub trait ToAD {
    fn to_ad(self) -> Arc<DangerousCell<Self>>
    where
        Self: Sized,
    {
        Arc::new(DangerousCell::new(self))
    }
}

impl<T> ToAD for T {}
