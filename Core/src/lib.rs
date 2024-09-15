#![feature(portable_simd)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::sync::Arc;
use mvutils::unsafe_utils::DangerousCell;

pub mod asset;
pub mod err;
pub mod input;
pub mod math;
pub mod render;
pub mod color;

pub trait ToAD {
    fn to_ad(self) -> Arc<DangerousCell<Self>> where Self: Sized {
        Arc::new(DangerousCell::new(self))
    }
}

impl<T> ToAD for T {}