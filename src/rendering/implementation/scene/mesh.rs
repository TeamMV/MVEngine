use crate::rendering::implementation::scene::material::Material;
use crate::utils::hashable::{Vec2, Vec3};

#[derive(Clone, Debug)]
pub struct MeshVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub material_id: u8,
}

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
}
