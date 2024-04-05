use std::ops::{Deref, DerefMut, Mul, MulAssign};
use std::simd::{f32x2, f32x4};

use mvutils::unsafe_utils::Unsafe;

#[derive(Default, Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Vec2(f32x2);

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self([x, y].into())
    }

    pub fn splat(val: f32) -> Self {
        Self([val; 2].into())
    }
}

#[repr(C)]
pub struct _DerefVec2 {
    pub x: f32,
    pub y: f32,
}

impl Deref for Vec2 {
    type Target = _DerefVec2;

    fn deref(&self) -> &Self::Target {
        unsafe { Unsafe::cast_ref(self) }
    }
}

impl DerefMut for Vec2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { Unsafe::cast_mut(self) }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
        }
    }

    pub fn splat(val: f32) -> Self {
        Self {
            x: val,
            y: val,
            z: val,
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Vec4(f32x4);

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self([x, y, z, w].into())
    }

    pub fn splat(val: f32) -> Self {
        Self([val; 4].into())
    }
}

impl Mul<f32> for Vec4 {
    type Output = Vec4;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * f32x4::from_array([rhs, rhs, rhs, rhs]))
    }
}

impl MulAssign<f32> for Vec4 {
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= f32x4::from_array([rhs, rhs, rhs, rhs])
    }
}

#[repr(C)]
pub struct _DerefVec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Deref for Vec4 {
    type Target = _DerefVec4;

    fn deref(&self) -> &Self::Target {
        unsafe { Unsafe::cast_ref(self) }
    }
}

impl DerefMut for Vec4 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { Unsafe::cast_mut(self) }
    }
}