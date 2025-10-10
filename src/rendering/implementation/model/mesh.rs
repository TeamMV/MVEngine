use crate::math::vec::{Vec2, Vec3};
use crate::rendering::implementation::model::material::Material;

pub struct MeshVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub material_id: u8,
}

pub struct Mesh {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
    pub materials: Vec<Material>,
}
