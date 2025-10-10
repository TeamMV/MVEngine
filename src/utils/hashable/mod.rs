use ordered_float::OrderedFloat;
use std::hash::Hash;

pub type Float = OrderedFloat<f32>;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HVec2 {
    pub x: Float,
    pub y: Float,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HVec3 {
    pub x: Float,
    pub y: Float,
    pub z: Float,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Vec4 {
    pub x: Float,
    pub y: Float,
    pub z: Float,
    pub w: Float,
}

impl HVec2 {
    #[inline]
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: Float::from(x),
            y: Float::from(y),
        }
    }
}

impl HVec3 {
    #[inline]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: Float::from(x),
            y: Float::from(y),
            z: Float::from(z),
        }
    }
}

impl Vec4 {
    #[inline]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            x: Float::from(x),
            y: Float::from(y),
            z: Float::from(z),
            w: Float::from(w),
        }
    }
}