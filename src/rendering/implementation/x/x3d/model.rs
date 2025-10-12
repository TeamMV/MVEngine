use crate::rendering::implementation::scene::mesh::Mesh;
use std::sync::Arc;

pub struct XLoadedModel {
    pub(crate) x_materials_version: u64,
    pub(crate) used_materials: Vec<u32>,
    pub(crate) mesh: Arc<Mesh>,
}
