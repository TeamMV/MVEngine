use std::iter::{Product};
use std::ops::{Add, Mul, MulAssign, AddAssign};
use num_traits::{Num, One, ToPrimitive, Zero};

pub mod mat;
pub mod quat;
pub mod vec;
pub mod curve;

pub trait Factorial {
    fn fact(&self) -> Self;
}

impl<T> Factorial for T where T: Num + MulAssign + AddAssign + Copy + PartialOrd {
    fn fact(&self) -> Self {
        let mut res = T::one();
        let mut i = T::one() + T::one();
        while i <= *self {
            res *= i;
            i += T::one();
        }
        res
    }
}

pub fn lerp<T>(near: T, far: T, p: f64) -> f64 where T: ToPrimitive  {
    near.to_f64().unwrap() + far.to_f64().unwrap() * p
}