use crate::math::vec::{DerefVec4, Vec4};
use mvutils::unsafe_utils::Unsafe;
use std::ops::{Deref, DerefMut, Mul};
use std::simd::{f32x4, simd_swizzle};

#[derive(Default, Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Quat(f32x4);

impl Quat {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self([x, y, z, w].into())
    }

    pub fn from_x(x: f32) -> Self {
        let (x, w) = (x * 0.5).sin_cos();
        Self::new(x, 0.0, 0.0, w)
    }

    pub fn from_y(y: f32) -> Self {
        let (y, w) = (y * 0.5).sin_cos();
        Self::new(0.0, y, 0.0, w)
    }

    pub fn from_z(z: f32) -> Self {
        let (z, w) = (z * 0.5).sin_cos();
        Self::new(0.0, 0.0, z, w)
    }

    pub fn from_euler(x: f32, y: f32, z: f32) -> Self {
        Self::from_x(x) * Self::from_y(y) * Self::from_z(z)
    }

    pub fn to_axes(&self) -> (Vec4, Vec4, Vec4) {
        let x2 = self.x + self.x;
        let y2 = self.y + self.y;
        let z2 = self.z + self.z;
        let xx = self.x * x2;
        let xy = self.x * y2;
        let xz = self.x * z2;
        let yy = self.y * y2;
        let yz = self.y * z2;
        let zz = self.z * z2;
        let wx = self.w * x2;
        let wy = self.w * y2;
        let wz = self.w * z2;

        let x = Vec4::new(1.0 - (yy + zz), xy + wz, xz - wy, 0.0);
        let y = Vec4::new(xy - wz, 1.0 - (xx + zz), yz + wx, 0.0);
        let z = Vec4::new(xz + wy, yz - wx, 1.0 - (xx + yy), 0.0);
        (x, y, z)
    }
}

impl Mul for Quat {
    type Output = Quat;

    fn mul(self, rhs: Self) -> Self::Output {
        let lhs = self.0;
        let rhs = rhs.0;

        let x = simd_swizzle!(lhs, [0, 0, 0, 0]);
        let y = simd_swizzle!(lhs, [1, 1, 1, 1]);
        let z = simd_swizzle!(lhs, [2, 2, 2, 2]);
        let w = simd_swizzle!(lhs, [3, 3, 3, 3]);

        let rhs_lw = w * rhs;
        let rhs_inverted = simd_swizzle!(rhs, [3, 2, 1, 0]);

        let rhs_inv_lx = x * rhs_inverted;
        let rhs_transformed = simd_swizzle!(rhs_inverted, [1, 0, 3, 2]);

        const PNPN: f32x4 = f32x4::from_array([1.0, -1.0, 1.0, -1.0]);
        let rhs_inv_lx_norm = rhs_inv_lx * PNPN;

        let rhs_trans_ly = y * rhs_transformed;
        let rhs_transformed_inverted = simd_swizzle!(rhs_transformed, [3, 2, 1, 0]);

        const PPNN: f32x4 = f32x4::from_array([1.0, 1.0, -1.0, -1.0]);
        let rhs_trans_ly_norm = rhs_trans_ly * PPNN;

        let rhs_trans_inv_lz = z * rhs_transformed_inverted;
        let sum_a = rhs_lw + rhs_inv_lx_norm;

        const NPPN: f32x4 = f32x4::from_array([-1.0, 1.0, 1.0, -1.0]);
        let rhs_trans_inv_lz_norm = rhs_trans_inv_lz * NPPN;
        let sum_b = rhs_trans_ly_norm + rhs_trans_inv_lz_norm;

        Self(sum_a + sum_b)
    }
}

impl Deref for Quat {
    type Target = DerefVec4;

    fn deref(&self) -> &Self::Target {
        unsafe { Unsafe::cast_ref(self) }
    }
}

impl DerefMut for Quat {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { Unsafe::cast_mut(self) }
    }
}
