use crate::rendering::implementation::model::material::Material;
use crate::rendering::implementation::model::mesh::Mesh;

pub mod material;
mod mesh;

pub struct Model {
    mesh: Mesh,
    materials: Vec<Material>
}