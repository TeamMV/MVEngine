use std::sync::Arc;
use crate::rendering::implementation::model::material::Material;
use crate::rendering::implementation::model::mesh::Mesh;

pub struct StandaloneModel {
    pub(crate) mesh: Arc<Mesh>,
    pub(crate) materials: Vec<Material>,
}

pub struct SceneModel {
    pub(crate) mesh: Arc<Mesh>,
    pub(crate) materials: Vec<u32>
}