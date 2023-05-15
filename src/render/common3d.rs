use glam::Vec3;
use crate::render::color::{Color, RGB};

pub struct Model {
    vert_count: u32,
    /// see: [`consts`]
    ///
    /// [`consts`]: crate::render::consts
    data: Vec<f32>,
    indices: Vec<u32>,
}

pub struct Material {}

pub struct Light {
    direction: Vec3,
    position: Vec<f32>,
    attenuation: f32,
    color: Color<RGB, f32>,
}