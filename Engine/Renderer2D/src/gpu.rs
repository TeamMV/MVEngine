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
pub struct Vertex {
    position: Vec3,
    rotation: Vec3,
    origin: Vec3,
    transform: CanvasTransform,
    color: Vec4,
    texture_id: f32,
    use_cam: f32
}

impl Vertex {
    pub fn get_attrib_desc() -> Vec<AttributeType> {
        vec![
            AttributeType::Float32x3,
            AttributeType::Float32x3,
            AttributeType::Float32x3,

            AttributeType::Float32x3,
            AttributeType::Float32x3,
            AttributeType::Float32x2,
            AttributeType::Float32x3,

            AttributeType::Float32x4,
            AttributeType::Float32,
            AttributeType::Float32
        ]
    }
}

#[repr(C)]
pub struct CanvasTransform {
    translation: Vec3,
    rotation: Vec3,
    scale: Vec2,
    origin: Vec3
}