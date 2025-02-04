use std::f32::consts::FRAC_PI_2;
use std::simd::{f32x16, f32x4};

use crate::math::quat::Quat;
use crate::math::vec::{Vec2, Vec3, Vec4};

#[derive(Default, Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Mat2(pub f32x4);

impl Mat2 {
    pub fn mul_vec(&self, vec: Vec2) -> Vec2 {
        let m11 = self.0[0];
        let m12 = self.0[1];
        let m21 = self.0[2];
        let m22 = self.0[3];

        Vec2 {
            x: m11 * vec.x + m12 * vec.y,
            y: m21 * vec.x + m22 * vec.y,
        }
    }

    pub fn as_slice(&self) -> &[f32] {
        self.0.as_array()
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Mat3 {
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3,
}

impl Mat3 {
    pub fn as_slice(&self) -> &[f32] {
        unsafe { std::slice::from_raw_parts(self as *const Mat3 as *const f32, 9) }
    }
}

#[derive(Default, Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Mat4(pub f32x16);

impl Mat4 {
    pub fn view(position: Vec4, rotation: Quat, scale: Vec4) -> Self {
        let (mut x, mut y, mut z) = rotation.to_axes();
        x *= scale.x;
        y *= scale.y;
        z *= scale.z;
        Self(
            [
                x.x, x.y, x.z, x.w, y.x, y.y, y.z, y.w, z.x, z.y, z.z, z.w, position.x, position.y,
                position.z, 1.0,
            ]
                .into(),
        )
    }

    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let inv_width = 1.0 / (right - left);
        let inv_height = 1.0 / (top - bottom);
        let inv_depth = 1.0 / (far - near);
        Self(
            [
                inv_width + inv_width,
                0.0,
                0.0,
                0.0,
                0.0,
                inv_height + inv_height,
                0.0,
                0.0,
                0.0,
                0.0,
                inv_depth,
                0.0,
                -(left + right) * inv_width,
                -(top + bottom) * inv_height,
                -inv_depth * near,
                1.0,
            ]
                .into(),
        )
    }

    pub fn perspective(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        let fov_tan = -(0.5 * fov + FRAC_PI_2).tan();
        let fov_ratio = fov_tan / aspect_ratio;
        let inv_depth = far / (far - near);
        Self(
            [
                fov_ratio,
                0.0,
                0.0,
                0.0,
                0.0,
                fov_tan,
                0.0,
                0.0,
                0.0,
                0.0,
                inv_depth,
                0.0,
                0.0,
                0.0,
                -inv_depth * near,
                0.0,
            ]
                .into(),
        )
    }

    pub fn as_slice(&self) -> &[f32] {
        self.0.as_array()
    }
}
