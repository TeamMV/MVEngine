use crate::rendering::api::MVTexture;
use crate::rendering::implementation::scene::material::Material;
use crate::rendering::implementation::scene::mesh::Mesh;
use std::sync::Arc;

pub struct StandaloneModel {
    pub(crate) mesh: Arc<Mesh>,
    pub(crate) materials: Vec<Material>,
    pub(crate) textures: Vec<MVTexture>,
}

#[derive(Debug)]
pub struct SceneModel {
    pub(crate) name: String,
    pub(crate) mesh: Arc<Mesh>,
    pub(crate) materials: Vec<u8>,
}
