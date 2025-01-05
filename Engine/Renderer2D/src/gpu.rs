use mvcore::math::mat::{Mat2, Mat4};
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

    pub fn apply_for_point(&self, point: (i32, i32)) -> (i32, i32) {
        let translated_x = point.0 as f32 - self.origin.x;
        let translated_y = point.1 as f32 - self.origin.y;
        let scaled_x = translated_x * self.scale.x;
        let scaled_y = translated_y * self.scale.y;
        let cos_theta = self.rotation.cos();
        let sin_theta = self.rotation.sin();
        let rotated_x = scaled_x * cos_theta - scaled_y * sin_theta;
        let rotated_y = scaled_x * sin_theta + scaled_y * cos_theta;
        let translated_x = rotated_x + self.origin.x + self.translation.x;
        let translated_y = rotated_y + self.origin.y + self.translation.y;
        (translated_x as i32, translated_y as i32)
    }
}
