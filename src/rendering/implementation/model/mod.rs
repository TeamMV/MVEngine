use crate::rendering::implementation::model::material::Material;
use crate::rendering::implementation::model::model::{SceneModel, StandaloneModel};

pub mod material;
pub mod mesh;
pub mod model;

pub struct Scene {
    models: Vec<SceneModel>,
    materials: Vec<Material>
}