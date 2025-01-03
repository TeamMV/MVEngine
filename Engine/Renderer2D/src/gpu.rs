use mvcore::math::mat::Mat4;
use mvcore::math::vec::{Vec2, Vec3, Vec4};
use mvcore::render::backend::pipeline::AttributeType;

#[repr(C)]
pub struct CameraBuffer {
    pub view_matrix: Mat4,
    pub proj_matrix: Mat4,
    pub screen_size: Vec2,
}

#[repr(C)]
#[derive(Clone)]
pub struct Transform {
    pub translation: Vec2,
    pub origin: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
    _align: u32,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            translation: Vec2::default(),
            origin: Vec2::default(),
            scale: Vec2::splat(1.0),
            rotation: 0.0,
            _align: 0,
        }
    }
}
