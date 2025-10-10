use std::sync::Arc;
use crate::rendering::implementation::model::mesh::Mesh;

pub struct XLoadedModel {
    pub(crate) x_materials_version: u64,
    pub(crate) used_materials: Vec<u32>,
    pub(crate) mesh: Arc<Mesh>
}