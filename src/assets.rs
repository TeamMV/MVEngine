use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use include_dir::*;

use crate::render::RenderCore;
use crate::render::shared::{EffectShader, Shader, Texture, TextureRegion};
use crate::render::text::{Font, FontLoader};

pub struct AssetManager {
    core: Option<Rc<RenderCore>>,
    font_loader: Option<FontLoader>,
    files: HashMap<String, File<'static>>,
    shaders: HashMap<String, Rc<RefCell<Shader>>>,
    effect_shaders: HashMap<String, Rc<RefCell<EffectShader>>>,
    textures: HashMap<String, Rc<RefCell<Texture>>>,
    texture_regions: HashMap<String, Rc<TextureRegion>>,
    fonts: HashMap<String, Rc<Font>>,
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
                files: file_map,
                shaders: HashMap::new(),
                effect_shaders: HashMap::new(),
                textures: HashMap::new(),
                texture_regions: HashMap::new(),
                fonts: HashMap::new(),
            },
        }
    }

    pub fn manual(dir: Dir<'static>) -> ManualAssetManager {
        ManualAssetManager {
            manager: AssetManager {
                core: None,
                font_loader: None,
                files: Self::map(dir),
                shaders: HashMap::new(),
                effect_shaders: HashMap::new(),
                textures: HashMap::new(),
                texture_regions: HashMap::new(),
                fonts: HashMap::new(),
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

pub struct AutomaticAssetManager {
    manager: AssetManager,
}

pub struct SemiAutomaticAssetManager {
    manager: AssetManager,
}

pub struct ManualAssetManager {
    manager: AssetManager,
}

crate::impl_am!(ManualAssetManager, SemiAutomaticAssetManager, AutomaticAssetManager);

#[macro_export]
macro_rules! impl_am {
    ($($t:ty),*) => {
        $(
            impl $t {
                pub fn set_render_core(&mut self, core: Rc<RenderCore>) {
                    self.manager.core = Some(core);
                    self.manager.font_loader = Some(FontLoader::new(self.manager.core.unwrap().clone()));
                }
            }
        )*
    };
}

pub trait ReadableAssetManager {
    fn get_shader(&self, id: &str) -> Rc<RefCell<Shader>>;
    fn try_get_shader(&self, id: &str) -> Option<Rc<RefCell<Shader>>>;
    fn get_effect_shader(&self, id: &str) -> Rc<RefCell<EffectShader>>;
    fn try_get_effect_shader(&self, id: &str) -> Option<Rc<RefCell<EffectShader>>>;
    fn get_texture(&self, id: &str) -> Rc<TextureRegion>;
    fn try_get_texture(&self, id: &str) -> Option<Rc<TextureRegion>>;
    fn get_font(&self, id: &str) -> Rc<Font>;
    fn try_get_font(&self, id: &str) -> Option<Rc<Font>>;
}

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
            }
        )*
    };
}

pub trait WritableAssetManager {
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
    fn prepare_texture(&mut self, id: &str, tex_id: &str);
    fn try_prepare_texture(&mut self, id: &str, tex_id: &str) -> Result<(), String>;
    fn crop_texture_region(&mut self, id: &str, tex_id: &str, x: u16, y: u16, width: u16, height: u16);
    fn try_crop_texture_region(&mut self, id: &str, tex_id: &str, x: u16, y: u16, width: u16, height: u16) -> Result<(), String>;
}

crate::impl_wam!(SemiAutomaticAssetManager, ManualAssetManager);

#[macro_export]
macro_rules! impl_wam {
    ($($t:ty),*) => {
        $(
            impl WritableAssetManager for $t {
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
                    let shader = self.manager.core.expect("Core not set!").create_shader(vertex_code.unwrap(), fragment_code.unwrap());
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
                    let shader = self.manager.core.expect("Core not set!").create_effect_shader(fragment_code.unwrap());
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
                    let texture = self.manager.core.expect("Core not set!").create_texture(texture.contents().to_vec());
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
                    let font = self.manager.font_loader.expect("Core not set!").load_ttf(ttf.contents().to_vec());
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

                    let font = self.manager.font_loader.expect("Core not set!").load_bitmap(image.contents().to_vec(), data_str.unwrap());
                    self.manager.fonts.insert(id.to_string(), Rc::new(font));
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

                fn crop_texture_region(&mut self, tex_id: &str, id: &str, x: u16, y: u16, width: u16, height: u16) {
                    if let Err(e) = self.try_crop_texture_region(tex_id, id, x, y, width, height) {
                        panic!("{}", e);
                    }
                }

                fn try_crop_texture_region(&mut self, tex_id: &str, id: &str, x: u16, y: u16, width: u16, height: u16) -> Result<(), String> {
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