use glam::Vec3;

use crate::old_render::color::{Color, RGB};

pub struct Light {
    pub(crate) position: Vec3,
    pub(crate) direcetion: Vec3,
    pub(crate) color: Color<RGB, f32>,
    pub(crate) attenuation: f32,
    pub(crate) cutoff: f32,
    pub(crate) radius: f32,
}