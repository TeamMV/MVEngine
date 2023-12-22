use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use mvutils::lazy;
use mvutils::once::Lazy;
use mvutils::utils::{Recover, RwUnchecked};

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
    ($name:ident, $t:ty, $mu:expr) => {
        impl_r!($name, $name, $t, $mu);
    };
    ($name:ident, $path:ident, $t:ty, $mu:expr) => {
        pub fn $name() -> Res<RwLock<$t>> {
            GLOBAL_RESOURCES.write().recover().$path.clone()
        }
    };
}

impl R {
    impl_r!(texture_regions, TextureRegion, true);
    #[cfg(feature = "3d")]
    impl_r!(models, Model);
    #[cfg(feature = "3d")]
    impl_r!(materials, Material, true);
    impl_r!(colors, Color<RGB, f32>, true);
    impl_r!(gradients, Gradient<RGB, f32>, true);

    pub(crate) fn convert_texture_core(texture: &str, id: String) {
        let res = GLOBAL_RESOURCES.write_unchecked();
        let tex = res.textures.get_core(texture);
        let region = TextureRegion::from(tex);
        res.texture_regions.register_core(id, Arc::new(RwLock::new(region)));
    }

    pub fn convert_texture(texture: &str, id: String) {
        let res = GLOBAL_RESOURCES.write_unchecked();
        let tex = res.textures.get(texture);
        let region = TextureRegion::from(tex);
        res.texture_regions.register(id, Arc::new(RwLock::new(region)));
    }

    pub(crate) fn crop_texture_core(
        texture: &str,
        id: String,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) {
        let res = GLOBAL_RESOURCES.write_unchecked();
        let tex = res.textures.get_core(texture);
        let region = TextureRegion::new(tex, x, y, width, height);
        res.texture_regions.register_core(id, Arc::new(RwLock::new(region)));
    }

    pub fn crop_texture(texture: &str, id: String, x: u32, y: u32, width: u32, height: u32) {
        let res = GLOBAL_RESOURCES.write_unchecked();
        let tex = res.textures.get(texture);
        let region = TextureRegion::new(tex, x, y, width, height);
        res.texture_regions.register(id, Arc::new(RwLock::new(region)));
    }
}

type Res<T> = Arc<Ref<T>>;
type MutRes<T> = Res<RwLock<T>>;

#[derive(Default)]
struct GlobalResources {
    textures: Res<Texture>,
    texture_regions: MutRes<TextureRegion>,
    #[cfg(feature = "3d")]
    models: Res<Model>,
    #[cfg(feature = "3d")]
    materials: MutRes<Material>,
    colors: MutRes<Color<RGB, f32>>,
    gradients: MutRes<Gradient<RGB, f32>>,
    //...
}

pub struct Ref<T> {
    map: RwLock<HashMap<String, Arc<T>>>,
}

impl<T> Ref<T> {
    pub fn get_core(&self, key: &str) -> Arc<T> {
        self.map
            .read_unchecked()
            .get(&("mvcore:".to_string() + key))
            .expect(&("Could not find resource with id \"".to_owned() + key + "\"."))
            .clone()
    }

    pub fn get(&self, key: &str) -> Arc<T> {
        self.map
            .read_unchecked()
            .get(&key.to_string())
            .expect(&("Could not find resource with id \"".to_owned() + key + "\"."))
            .clone()
    }

    pub fn register_core(&self, key: String, res: Arc<T>) {
        self.map
            .write_unchecked()
            .insert("mvcore:".to_string() + &*key, res);
    }

    pub fn register(&self, key: String, res: Arc<T>) {
        self.map.write_unchecked().insert(key, res);
    }
}

impl<T> Default for Ref<T> {
    fn default() -> Self {
        Self {
            map: HashMap::new().into(),
        }
    }
}
