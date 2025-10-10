use std::sync::Arc;
use crate::rendering::implementation::model::material::Material;
use crate::rendering::implementation::model::mesh::Mesh;

pub mod material;
pub mod mesh;

pub struct Model {
    pub(crate) mesh: Arc<Mesh>,
    pub(crate) materials: Vec<Material>,
}
