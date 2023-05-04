use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use include_dir::*;
use mvutils::sealable;
use crate::old_render::model::{Model, ModelLoader};

use crate::old_render::RenderCore;
use crate::old_render::shared::{EffectShader, Shader, Texture, TextureRegion};
use crate::old_render::text::{Font, FontLoader};

sealable!();

pub struct AssetManager {
    core: Option<Arc<RenderCore>>,
    font_loader: Option<FontLoader>,
    model_loader: Option<ModelLoader>,
    pub(crate) files: HashMap<String, File<'static>>,
    shaders: HashMap<String, Rc<RefCell<Shader>>>,
    effect_shaders: HashMap<String, Rc<RefCell<EffectShader>>>,
    pub(crate) textures: HashMap<String, Rc<RefCell<Texture>>>,
    texture_regions: HashMap<String, Rc<TextureRegion>>,
    fonts: HashMap<String, Rc<Font>>,
    models: HashMap<String, Rc<RefCell<Model>>>,
}

impl AssetManager {
    pub fn automatic(dir: Dir<'static>) -> AutomaticAssetManager {
        let config = dir.get_file("assets.dat").expect("Automatic asset manager requires assets.dat file!").clone();
        let mut file_map = Self::map(dir);
        //parse config, map files to assets
        drop(file_map);
        todo!()
    }

    pub fn semi_automatic(dir: Dir<'static>) -> SemiAutomaticAssetManager {
        let mut config = dir.get_file("assets.dat").cloned();
        let mut file_map = Self::map(dir);
        //parse config, map files to assets
        //todo!()
        SemiAutomaticAssetManager {
            manager: AssetManager {
                core: None,
                font_loader: None,
                model_loader: None,
                files: file_map,
                shaders: HashMap::new(),
                effect_shaders: HashMap::new(),
                textures: HashMap::new(),
                texture_regions: HashMap::new(),
                fonts: HashMap::new(),
                models: HashMap::new(),
            },
        }
    }

    pub fn manual(dir: Dir<'static>) -> ManualAssetManager {
        ManualAssetManager {
            manager: AssetManager {
                core: None,
                font_loader: None,
                model_loader: None,
                files: Self::map(dir),
                shaders: HashMap::new(),
                effect_shaders: HashMap::new(),
                textures: HashMap::new(),
                texture_regions: HashMap::new(),
                fonts: HashMap::new(),
                models: HashMap::new(),
            },
        }
    }

    fn map(dir: Dir) -> HashMap<String, File> {
        let mut file_map: HashMap<String, File> = HashMap::new();
        Self::files_deep(dir).into_iter().map(|file| {
            (file.path().to_path_buf().into_os_string().into_string().unwrap(), file)
        }).for_each(|(pair, file)| {
            file_map.insert(pair, file);
        });
        file_map
    }

    fn files_deep(dir: Dir) -> Vec<File> {
        let mut files_deep = dir
            .dirs()
            .map(ToOwned::to_owned)
            .flat_map(Self::files_deep)
            .collect::<Vec<_>>();
        files_deep
            .append(dir
                .files()
                .map(File::to_owned)
                .collect::<Vec<_>>().as_mut());
        files_deep
    }
}

seal!(
    pub struct AutomaticAssetManager {
        pub(crate) manager: AssetManager,
    }
);

seal!(
    pub struct SemiAutomaticAssetManager {
        pub(crate) manager: AssetManager,
    }
);

seal!(
    pub struct ManualAssetManager {
        pub(crate) manager: AssetManager,
    }
);

crate::impl_am!(ManualAssetManager, SemiAutomaticAssetManager, AutomaticAssetManager);

#[macro_export]
macro_rules! impl_am {
    ($($t:ty),*) => {
        $(
            impl $t {
                pub fn set_render_core(&mut self, core: Arc<RenderCore>) {
                    self.manager.core = Some(core);
                    self.manager.font_loader = Some(FontLoader::new(self.manager.core.clone().unwrap()));
                    self.manager.model_loader = Some(ModelLoader::new(self.manager.core.clone().unwrap(), &mut self.manager));
                }
            }
        )*
    };
}

sealed!(
    pub trait ReadableAssetManager {
        fn get_shader(&self, id: &str) -> Rc<RefCell<Shader>>;
        fn try_get_shader(&self, id: &str) -> Option<Rc<RefCell<Shader>>>;
        fn get_effect_shader(&self, id: &str) -> Rc<RefCell<EffectShader>>;
        fn try_get_effect_shader(&self, id: &str) -> Option<Rc<RefCell<EffectShader>>>;
        fn get_texture(&self, id: &str) -> Rc<TextureRegion>;
        fn try_get_texture(&self, id: &str) -> Option<Rc<TextureRegion>>;
        fn get_font(&self, id: &str) -> Rc<Font>;
        fn try_get_font(&self, id: &str) -> Option<Rc<Font>>;
        fn get_model(&self, id: &str) -> Rc<RefCell<Model>>;
        fn try_get_model(&self, id: &str) -> Option<Rc<RefCell<Model>>>;
    }
);

crate::impl_ram!(AutomaticAssetManager, SemiAutomaticAssetManager, ManualAssetManager);

#[macro_export]
macro_rules! impl_ram {
    ($($t:ty),*) => {
        $(
            impl ReadableAssetManager for $t {
                fn get_shader(&self, id: &str) -> Rc<RefCell<Shader>> {
                    self.manager.shaders.get(id).unwrap().clone()
                }

                fn try_get_shader(&self, id: &str) -> Option<Rc<RefCell<Shader>>> {
                    self.manager.shaders.get(id).cloned()
                }

                fn get_effect_shader(&self, id: &str) -> Rc<RefCell<EffectShader>> {
                    self.manager.effect_shaders.get(id).unwrap().clone()
                }

                fn try_get_effect_shader(&self, id: &str) -> Option<Rc<RefCell<EffectShader>>> {
                    self.manager.effect_shaders.get(id).cloned()
                }

                fn get_texture(&self, id: &str) -> Rc<TextureRegion> {
                    self.manager.texture_regions.get(id).unwrap().clone()
                }

                fn try_get_texture(&self, id: &str) -> Option<Rc<TextureRegion>> {
                    self.manager.texture_regions.get(id).cloned()
                }

                fn get_font(&self, id: &str) -> Rc<Font> {
                    self.manager.fonts.get(id).unwrap().clone()
                }

                fn try_get_font(&self, id: &str) -> Option<Rc<Font>> {
                    self.manager.fonts.get(id).cloned()
                }

                fn get_model(&self, id: &str) -> Rc<RefCell<Model>> {
                    self.manager.models.get(id).unwrap().clone()
                }

                fn try_get_model(&self, id: &str) -> Option<Rc<RefCell<Model>>> {
                    self.manager.models.get(id).cloned()
                }
            }
        )*
    };
}

sealed!(
    pub trait WritableAssetManager {
        fn get_contents(&mut self, path: &str) -> Vec<u8>;
        fn try_get_contents(&mut self, path: &str) -> Result<Vec<u8>, String>;
        fn get_string_contents(&mut self, path: &str) -> String;
        fn try_get_string_contents(&mut self, path: &str) -> Result<String, String>;
        fn load_shader(&mut self, id: &str, vertex_path: &str, fragment_path: &str);
        fn try_load_shader(&mut self, id: &str, vertex_path: &str, fragment_path: &str) -> Result<(), String>;
        fn load_effect_shader(&mut self, id: &str, fragment_path: &str);
        fn try_load_effect_shader(&mut self, id: &str, fragment_path: &str) -> Result<(), String>;
        fn load_texture(&mut self, id: &str, texture_path: &str);
        fn try_load_texture(&mut self, id: &str, texture_path: &str) -> Result<(), String>;
        fn load_ttf_font(&mut self, id: &str, ttf_path: &str);
        fn try_load_ttf_font(&mut self, id: &str, ttf_path: &str) -> Result<(), String>;
        fn load_bitmap_font(&mut self, id: &str, bitmap_path: &str, data_path: &str);
        fn try_load_bitmap_font(&mut self, id: &str, bitmap_path: &str, data_path: &str) -> Result<(), String>;
        fn load_model(&mut self, id: &str, model_path: &str);
        fn try_load_model(&mut self, id: &str, model_path: &str) -> Result<(), String>;
        fn prepare_texture(&mut self, id: &str, tex_id: &str);
        fn try_prepare_texture(&mut self, id: &str, tex_id: &str) -> Result<(), String>;
        fn crop_texture_region(&mut self, id: &str, tex_id: &str, x: u32, y: u32, width: u32, height: u32);
        fn try_crop_texture_region(&mut self, id: &str, tex_id: &str, x: u32, y: u32, width: u32, height: u32) -> Result<(), String>;
    }
);

crate::impl_wam!(SemiAutomaticAssetManager, ManualAssetManager);

#[macro_export]
macro_rules! impl_wam {
    ($($t:ty),*) => {
        $(
            impl WritableAssetManager for $t {
                fn get_contents(&mut self, path: &str) -> Vec<u8> {
                    let contents = self.try_get_contents(path);
                    if contents.is_err() {
                        panic!("{}", contents.unwrap_err());
                    }
                    contents.unwrap()
                }

                fn try_get_contents(&mut self, path: &str) -> Result<Vec<u8>, String> {
                    let file = self.manager.files.remove(path);
                    if file.is_none() {
                        return Err(format!("File not found: {}", path));
                    }
                    Ok(file.unwrap().contents().to_vec())
                }

                fn get_string_contents(&mut self, path: &str) -> String {
                    let contents = self.try_get_string_contents(path);
                    if contents.is_err() {
                        panic!("{}", contents.unwrap_err());
                    }
                    contents.unwrap()
                }

                fn try_get_string_contents(&mut self, path: &str) -> Result<String, String> {
                    let file = self.manager.files.remove(path);
                    if file.is_none() {
                        return Err(format!("File not found: {}", path));
                    }
                    let contents = file.unwrap();
                    let contents = contents.contents_utf8();
                    if contents.is_none() {
                        return Err(format!("Contents are not a utf-8 compatible string in file: {}", path));
                    }
                    Ok(contents.unwrap().to_string())
                }

                fn load_shader(&mut self, id: &str, vertex_path: &str, fragment_path: &str) {
                    if let Err(e) = self.try_load_shader(id, vertex_path, fragment_path) {
                        panic!("{}", e);
                    }
                }

                fn try_load_shader(&mut self, id: &str, vertex_path: &str, fragment_path: &str) -> Result<(), String> {
                    let vertex = self.manager.files.remove(vertex_path);
                    if vertex.is_none() {
                        return Err(format!("Vertex file {} not found!", vertex_path));
                    }
                    let vertex = vertex.unwrap();
                    let fragment = self.manager.files.remove(fragment_path);
                    if fragment.is_none() {
                        self.manager.files.insert(vertex_path.to_string(), vertex);
                        return Err(format!("Fragment file {} not found!", fragment_path));
                    }
                    let fragment = fragment.unwrap();
                    let vertex_code = vertex.contents_utf8();
                    let fragment_code = fragment.contents_utf8();
                    if vertex_code.is_none() {
                        self.manager.files.insert(vertex_path.to_string(), vertex);
                        self.manager.files.insert(fragment_path.to_string(), fragment);
                        return Err(format!("Illegal vertex code in file {}!", vertex_path));
                    }
                    if fragment_code.is_none() {
                        self.manager.files.insert(vertex_path.to_string(), vertex);
                        self.manager.files.insert(fragment_path.to_string(), fragment);
                        return Err(format!("Illegal fragment code in file {}!", fragment_path));
                    }
                    let shader = self.manager.core.clone().expect("Core not set!").create_shader(vertex_code.unwrap(), fragment_code.unwrap());
                    self.manager.shaders.insert(id.to_string(), Rc::new(RefCell::new(shader)));
                    Ok(())
                }

                fn load_effect_shader(&mut self, id: &str, fragment_path: &str) {
                    if let Err(e) = self.try_load_effect_shader(id, fragment_path) {
                        panic!("{}", e);
                    }
                }

                fn try_load_effect_shader(&mut self, id: &str, fragment_path: &str) -> Result<(), String> {
                    let fragment = self.manager.files.remove(fragment_path);
                    if fragment.is_none() {
                        return Err(format!("Fragment file {} not found!", fragment_path));
                    }
                    let fragment = fragment.unwrap();
                    let fragment_code = fragment.contents_utf8();
                    if fragment_code.is_none() {
                        self.manager.files.insert(fragment_path.to_string(), fragment);
                        return Err(format!("Illegal fragment code in file {}!", fragment_path));
                    }
                    let shader = self.manager.core.clone().expect("Core not set!").create_effect_shader(fragment_code.unwrap());
                    self.manager.effect_shaders.insert(id.to_string(), Rc::new(RefCell::new(shader)));
                    Ok(())
                }

                fn load_texture(&mut self, id: &str, texture_path: &str) {
                    if let Err(e) = self.try_load_texture(id, texture_path) {
                        panic!("{}", e);
                    }
                }

                fn try_load_texture(&mut self, id: &str, texture_path: &str) -> Result<(), String> {
                    let texture = self.manager.files.remove(texture_path);
                    if texture.is_none() {
                        return Err(format!("Texture file {} not found!", texture_path));
                    }
                    let texture = texture.unwrap();
                    let texture = self.manager.core.clone().expect("Core not set!").create_texture(texture.contents());
                    self.manager.textures.insert(id.to_string(), Rc::new(RefCell::new(texture)));
                    Ok(())
                }

                fn load_ttf_font(&mut self, id: &str, ttf_path: &str) {
                    if let Err(e) = self.try_load_ttf_font(id, ttf_path) {
                        panic!("{}", e);
                    }
                }

                fn try_load_ttf_font(&mut self, id: &str, ttf_path: &str) -> Result<(), String> {
                    let ttf = self.manager.files.remove(ttf_path);
                    if ttf.is_none() {
                        return Err(format!("Ttf file {} not found!", ttf_path));
                    }
                    let ttf = ttf.unwrap();
                    let font = self.manager.font_loader.as_ref().expect("Core not set!").load_ttf(ttf.contents().to_vec());
                    self.manager.fonts.insert(id.to_string(), Rc::new(font));
                    Ok(())
                }

                fn load_bitmap_font(&mut self, id: &str, bitmap_path: &str, data_path: &str){
                    if let Err(e) = self.try_load_bitmap_font(id, bitmap_path, data_path) {
                        panic!("{}", e);
                    }
                }

                fn try_load_bitmap_font(&mut self, id: &str, bitmap_path: &str, data_path: &str) -> Result<(), String> {
                    let image = self.manager.files.remove(bitmap_path);
                    if image.is_none() {
                        return Err(format!("Bitmap file {} not found!", bitmap_path));
                    }
                    let image = image.unwrap();
                    let data = self.manager.files.remove(data_path);
                    if data.is_none() {
                        self.manager.files.insert(bitmap_path.to_string(), image);
                        return Err(format!("Data file {} not found!", data_path));
                    }
                    let data = data.unwrap();
                    let data_str = data.contents_utf8();
                    if data_str.is_none() {
                        self.manager.files.insert(bitmap_path.to_string(), image);
                        self.manager.files.insert(data_path.to_string(), data);
                        return Err(format!("Illegal data in file {}!", data_path));
                    }
                    let font = self.manager.font_loader.as_ref().expect("Core not set!").load_bitmap(image.contents(), data_str.unwrap());
                    self.manager.fonts.insert(id.to_string(), Rc::new(font));
                    Ok(())
                }

                fn load_model(&mut self, id: &str, model_path: &str) {
                    if let Err(e) = self.try_load_model(id, model_path) {
                        panic!("{}", e);
                    }
                }

                fn try_load_model(&mut self, id: &str, model_path: &str) -> Result<(), String> {
                    let model = self.manager.files.remove(model_path);
                    if model.is_none() {
                        return Err(format!("Model file {} not found!", model_path));
                    }
                    let model = model.unwrap();
                    let model = self.manager.model_loader.as_ref().expect("Core not set!").load_model(model_path, model);
                    self.manager.models.insert(id.to_string(), Rc::new(RefCell::new(model)));
                    Ok(())
                }

                fn prepare_texture(&mut self, tex_id: &str, id: &str) {
                    if let Err(e) = self.try_prepare_texture(tex_id, id) {
                        panic!("{}", e);
                    }
                }

                fn try_prepare_texture(&mut self, tex_id: &str, id: &str) -> Result<(), String> {
                    let texture = self.manager.textures.get(tex_id);
                    if texture.is_none() {
                        return Err(format!("Texture {} not found!", tex_id));
                    }
                    let region = TextureRegion::from(texture.unwrap().clone());
                    self.manager.texture_regions.insert(id.to_string(), Rc::new(region));
                    Ok(())
                }

                fn crop_texture_region(&mut self, tex_id: &str, id: &str, x: u32, y: u32, width: u32, height: u32) {
                    if let Err(e) = self.try_crop_texture_region(tex_id, id, x, y, width, height) {
                        panic!("{}", e);
                    }
                }

                fn try_crop_texture_region(&mut self, tex_id: &str, id: &str, x: u32, y: u32, width: u32, height: u32) -> Result<(), String> {
                    let texture = self.manager.textures.get(tex_id);
                    if texture.is_none() {
                        return Err(format!("Texture {} not found!", tex_id));
                    }
                    let region = TextureRegion::new(texture.unwrap().clone(), x, y, width, height);
                    self.manager.texture_regions.insert(id.to_string(), Rc::new(region));
                    Ok(())
                }
            }
        )*
    }
}

pub enum AssetType {
    Texture,
    Sound,
    Model,
    Script,
    Shader,
    Font,
    Config,
    Other,
}