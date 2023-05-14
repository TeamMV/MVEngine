use glam::Vec3;
use crate::render::color::{Color, RGB};

pub struct Model {
    vert_count: u32,
    data: Vec<f32>,///@see: consts.rs
    indices: Vec<u32>,
}

pub struct Material {}

pub struct Light {
    direction: Vec3,
    position: Vec<3>,
    attenuation: f32,
    color: Color<RGB, f32>,
}