use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use mvutils::lazy;
use mvutils::once::Lazy;
use mvutils::utils::Recover;

#[cfg(feature = "gui")]
use crate::gui::Gui;
use crate::render::color::{Color, Gradient, RGB};
use crate::render::common::{Texture, TextureRegion};
#[cfg(feature = "3d")]
use crate::render::common3d::{Material, Model};

lazy! {
    static GLOBAL_RESOURCES: RwLock<GlobalResources> = GlobalResources::default().into();
}

pub struct R;

macro_rules! impl_r {
    ($name:ident, $t:ty) => {
        impl_r!($name, $name, $t);
    };
    ($name:ident, $path:ident, $t:ty) => {
        pub fn $name() -> Res<$t> {
            GLOBAL_RESOURCES.write().recover().$path.clone()
        }
    };
}

impl R {
    impl_r!(texture_regions, TextureRegion);
    #[cfg(feature = "3d")]
    impl_r!(models, Model);
    #[cfg(feature = "3d")]
    impl_r!(materials, Material);
    impl_r!(colors, Color<RGB, f32>);
    impl_r!(gradients, Gradient<RGB, f32>);
    #[cfg(feature = "gui")]
    impl_r!(guis, Gui);

    pub(crate) fn convert_texture_core(texture: &str, id: String) {
        let res = GLOBAL_RESOURCES.write().recover();
        let tex = res.textures.get_core(texture);
        let region = TextureRegion::from(tex);
        res.texture_regions.register_core(id, Arc::new(region));
    }

    pub fn convert_texture(texture: &str, id: String) {
        let res = GLOBAL_RESOURCES.write().recover();
        let tex = res.textures.get(texture);
        let region = TextureRegion::from(tex);
        res.texture_regions.register(id, Arc::new(region));
    }

    pub(crate) fn crop_texture_core(
        texture: &str,
        id: String,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) {
        let res = GLOBAL_RESOURCES.write().recover();
        let tex = res.textures.get_core(texture);
        let region = TextureRegion::new(tex, x, y, width, height);
        res.texture_regions.register_core(id, Arc::new(region));
    }

    pub fn crop_texture(texture: &str, id: String, x: u32, y: u32, width: u32, height: u32) {
        let res = GLOBAL_RESOURCES.write().recover();
        let tex = res.textures.get(texture);
        let region = TextureRegion::new(tex, x, y, width, height);
        res.texture_regions.register(id, Arc::new(region));
    }
}

type Res<T> = Arc<Ref<T>>;

#[derive(Default)]
struct GlobalResources {
    textures: Res<Texture>,
    texture_regions: Res<TextureRegion>,
    #[cfg(feature = "3d")]
    models: Res<Model>,
    #[cfg(feature = "3d")]
    materials: Res<Material>,
    colors: Res<Color<RGB, f32>>,
    gradients: Res<Gradient<RGB, f32>>,
    #[cfg(feature = "gui")]
    guis: Res<Gui>,
    //...
}

pub struct Ref<T> {
    map: RwLock<HashMap<String, Arc<T>>>,
}

impl<T> Ref<T> {
    pub fn get_core(&self, key: &str) -> Arc<T> {
        self.map
            .read()
            .recover()
            .get(&("mvcore:".to_string() + key))
            .expect(&("Could not find resource with id \"".to_owned() + key + "\"."))
            .clone()
    }

    pub fn get(&self, key: &str) -> Arc<T> {
        self.map
            .read()
            .recover()
            .get(&key.to_string())
            .expect(&("Could not find resource with id \"".to_owned() + key + "\"."))
            .clone()
    }

    pub fn register_core(&self, key: String, res: Arc<T>) {
        self.map
            .write()
            .recover()
            .insert("mvcore:".to_string() + &*key, res);
    }

    pub fn register(&self, key: String, res: Arc<T>) {
        self.map.write().recover().insert(key, res);
    }
}

impl<T> Default for Ref<T> {
    fn default() -> Self {
        Self {
            map: HashMap::new().into(),
        }
    }
}
