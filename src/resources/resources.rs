use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use mvutils::once::Lazy;
use mvutils::lazy;
use mvutils::utils::Recover;
use crate::render::color::{Color, Gradient, RGB};
use crate::render::common3d::{Material, Model};
use crate::render::common::{Texture, TextureRegion};
lazy! {
    static GLOBAL_RESOURCES: RwLock<GlobalResources> = GlobalResources::default().into();
}

pub struct R;

macro_rules! impl_R {
    ($name:ident, $t:ty) => {
        impl_R!($name, $name, $t);
    };
    ($name:ident, $path:ident, $t:ty) => {
        pub fn $name() -> Arc<Ref<$t>> {
            GLOBAL_RESOURCES.read().recover().$path.clone()
        }
    };
}

impl R {
    impl_R!(textures, texture_regions, TextureRegion);
    impl_R!(models, Model);
    impl_R!(materials, Material);
    impl_R!(colors, Color<RGB, f32>);
    impl_R!(gradients, Gradient<RGB, f32>);

    pub(crate) fn convert_texture_core(texture: &str, id: String) {
        let res = GLOBAL_RESOURCES.write().recover();
        let tex = res.textures.get_core(texture);
        let region = TextureRegion::from(tex);
        res.texture_regions.insert_core(id, region);
    }

    pub fn convert_texture(texture: &str, id: String) {
        let res = GLOBAL_RESOURCES.write().recover();
        let tex = res.textures.get(texture);
        let region = TextureRegion::from(tex);
        res.texture_regions.insert(id, region);
    }

    pub(crate) fn crop_texture_core(texture: &str, id: String, x: u32, y: u32, width: u32, height: u32) {
        let res = GLOBAL_RESOURCES.write().recover();
        let tex = res.textures.get_core(texture);
        let region = TextureRegion::new(tex, x, y, width, height);
        res.texture_regions.insert_core(id, region);
    }

    pub fn crop_texture(texture: &str, id: String, x: u32, y: u32, width: u32, height: u32) {
        let res = GLOBAL_RESOURCES.write().recover();
        let tex = res.textures.get(texture);
        let region = TextureRegion::new(tex, x, y, width, height);
        res.texture_regions.insert(id, region);
    }
}

#[derive(Default)]
struct GlobalResources {
    textures: Arc<Ref<Texture>>,
    texture_regions: Arc<Ref<TextureRegion>>,
    models: Arc<Ref<Model>>,
    materials: Arc<Ref<Material>>,
    colors: Arc<Ref<Color<RGB, f32>>>,
    gradients: Arc<Ref<Gradient<RGB, f32>>>,

    //...
}

#[derive(Default)]
pub struct Ref<T> {
    map: HashMap<String, Arc<T>>
}

impl<T> Ref<T> {
    pub fn get_core(&self, key: &str) -> Arc<T> {
        self.map.get( &"mvcore:".to_string() + key).expect("Could not find resource with id \"" + key.as_str() + "\".").clone()
    }

    pub fn get(&self, key: &str) -> Arc<T> {
        self.map.get(&key.to_string()).expect("Could not find resource with id \"" + key.as_str() + "\".").clone()
    }

    pub(crate) fn register_core(&mut self, key: String, res: Arc<T>) {
        self.map.insert("mvcore:".to_string() + &*key, res);
    }

    pub(crate) fn register(&mut self, key: String, res: Arc<T>) {
        self.map.insert(key, res);
    }
}