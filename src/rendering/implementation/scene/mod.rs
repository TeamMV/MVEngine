use crate::rendering::api::MVTexture;
use crate::rendering::implementation::scene::material::Material;
use crate::rendering::implementation::scene::model::{SceneModel, StandaloneModel};

pub mod material;
pub mod mesh;
pub mod model;

#[derive(Debug)]
pub struct Scene {
    pub(crate) models: Vec<SceneModel>,
    pub(crate) materials: Vec<Material>,
    pub(crate) textures: Vec<MVTexture>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            models: vec![],
            materials: vec![],
            textures: vec![],
        }
    }
}
