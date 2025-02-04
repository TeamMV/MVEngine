use std::fmt::{Debug, Formatter, Write};
use std::ops::{Deref, DerefMut, Mul, MulAssign};
use std::simd::f32x4;

use mvutils::unsafe_utils::Unsafe;
use crate::math::mat::Mat3;

#[derive(Default, Copy, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn mul_xy(&self, x: f32, y: f32) -> Vec2 {
        Vec2::new(self.x * x, self.y * y)
    }

    pub fn as_i32_tuple(&self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }

    pub fn as_slice(&self) -> &[f32] {
        unsafe { std::slice::from_raw_parts(self as *const Vec2 as *const f32, 2) }
    }
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn splat(val: f32) -> Self {
        Self { x: val, y: val }
    }
}

impl Debug for Vec2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char('(')?;
        f.write_str(format!("{}, {}", self.x, self.y).as_str())?;
        f.write_char(')')
    }
}

#[derive(Default, Debug, Copy, Clone)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn splat(val: f32) -> Self {
        Self {
            x: val,
            y: val,
            z: val,
        }
    }

    pub fn to_radians(self) -> Self {
        Self {
            x: self.x.to_radians(),
            y: self.y.to_radians(),
            z: self.z.to_radians(),
        }
    }

    pub fn to_degrees(self) -> Self {
        Self {
            x: self.x.to_degrees(),
            y: self.y.to_degrees(),
            z: self.z.to_degrees(),
        }
    }

    pub fn as_slice(&self) -> &[f32] {
        unsafe { std::slice::from_raw_parts(self as *const Vec3 as *const f32, 3) }
    }
}

impl Into<Vec4> for Vec3 {
    fn into(self) -> Vec4 {
        Vec4::new(self.x, self.y, self.z, 0.0)
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

    pub fn as_slice(&self) -> &[f32] {
        self.0.as_array()
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
pub struct DerefVec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Deref for Vec4 {
    type Target = DerefVec4;

    fn deref(&self) -> &Self::Target {
        unsafe { Unsafe::cast_ref(self) }
    }
}

impl DerefMut for Vec4 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { Unsafe::cast_mut(self) }
    }
}
