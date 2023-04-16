use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use include_dir::*;
use crate::render::RenderCore;

use crate::render::shared::{Shader, Texture, TextureRegion};

pub struct AssetManager {
    files: HashMap<String, File<'static>>,
    shaders: HashMap<String, Shader>,
    textures: HashMap<String, Texture>,
    texture_regions: HashMap<String, TextureRegion>,
}

impl AssetManager {
    pub fn automatic(dir: Dir) -> AutomaticAssetManager {
        let config = dir.get_file("assets.dat").expect("Automatic asset manager requires assets.dat file!").clone();
        let mut file_map = Self::map(dir);
        //parse config, map files to assets
        drop(file_map);
        todo!()
    }

    pub fn new(dir: Dir) -> SemiAutomaticAssetManager {
        let config = dir.get_file("assets.dat").map(|f| f.clone());
        let mut file_map = Self::map(dir);
        //parse config, map files to assets
        todo!()
    }

    pub fn manual(dir: Dir) -> ManualAssetManager {
        ManualAssetManager {
            manager: AssetManager {
                files: Self::map(dir),
                shaders: HashMap::new(),
                textures: HashMap::new(),
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
                .into_iter()
                .map(File::to_owned)
                .collect::<Vec<_>>().as_mut());
        files_deep
    }
}

pub struct AutomaticAssetManager {
    manager: AssetManager,
}

impl AutomaticAssetManager {

}

pub struct SemiAutomaticAssetManager {
    manager: AssetManager,
}

impl SemiAutomaticAssetManager {

}

pub struct ManualAssetManager {
    manager: AssetManager,
}

impl ManualAssetManager {

}

pub trait ReadableAssetManager {
    fn get_shader(&self, id: &str) -> &Shader;
    fn get_shader_mut(&mut self, id: &str) -> &mut Shader;
    fn try_get_shader(&self, id: &str) -> Option<&Shader>;
    fn try_get_shader_mut(&mut self, id: &str) -> Option<&mut Shader>;
    //fn get_texture(&self, id: &str) -> &Texture;
    //fn get_texture_mut(&mut self, id: &str) -> &mut Texture;
    //fn try_get_texture(&self, id: &str) -> Option<&Texture>;
    //fn try_get_texture_mut(&mut self, id: &str) -> Option<&mut Texture>;
    fn get_texture(&self, id: &str) -> &TextureRegion;
    fn get_texture_mut(&mut self, id: &str) -> &mut TextureRegion;
    fn try_get_texture(&self, id: &str) -> Option<&TextureRegion>;
    fn try_get_texture_mut(&mut self, id: &str) -> Option<&mut TextureRegion>;
}

crate::impl_ram!(AutomaticAssetManager, SemiAutomaticAssetManager, ManualAssetManager);

#[macro_export]
macro_rules! impl_ram {
    ($($t:ty),*) => {
        $(
            impl ReadableAssetManager for $t {
                fn get_shader(&self, id: &str) -> &Shader {
                    self.manager.shaders.get(id).unwrap()
                }

                fn get_shader_mut(&mut self, id: &str) -> &mut Shader {
                    self.manager.shaders.get_mut(id).unwrap()
                }

                fn try_get_shader(&self, id: &str) -> Option<&Shader> {
                    self.manager.shaders.get(id)
                }

                fn try_get_shader_mut(&mut self, id: &str) -> Option<&mut Shader> {
                    self.manager.shaders.get_mut(id)
                }

                fn get_texture(&self, id: &str) -> &TextureRegion {
                    self.manager.texture_regions.get(id).unwrap()
                }

                fn get_texture_mut(&mut self, id: &str) -> &mut TextureRegion {
                    self.manager.texture_regions.get_mut(id).unwrap()
                }

                fn try_get_texture(&self, id: &str) -> Option<&TextureRegion> {
                    self.manager.texture_regions.get(id)
                }

                fn try_get_texture_mut(&mut self, id: &str) -> Option<&mut TextureRegion> {
                    self.manager.texture_regions.get_mut(id)
                }
            }
        )*
    };
}

pub trait WritableAssetManager {
    fn load_shader(&mut self, render_core: &RenderCore, id: &str, vertex_path: &str, fragment_path: &str);
    fn try_load_shader(&mut self, render_core: &RenderCore, id: &str, vertex_path: &str, fragment_path: &str) -> Result<(), String>;
    fn load_texture(&mut self, render_core: &RenderCore, id: &str, texture_path: &str);
    fn try_load_texture(&mut self, render_core: &RenderCore, id: &str, texture_path: &str) -> Result<(), String>;
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
                fn load_shader(&mut self, render_core: &RenderCore, id: &str, vertex_path: &str, fragment_path: &str) {
                    if let Err(e) = self.try_load_shader(render_core, id, vertex_path, fragment_path) {
                        panic!(e);
                    }
                }

                fn try_load_shader(&mut self, render_core: &RenderCore, id: &str, vertex_path: &str, fragment_path: &str) -> Result<(), String> {
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
                    let shader = render_core.create_shader(vertex_code.unwrap(), fragment_code.unwrap());
                    self.manager.shaders.insert(id.to_string(), shader);
                    Ok(())
                }

                fn load_texture(&mut self, render_core: &RenderCore, id: &str, texture_path: &str) {
                    if let Err(e) = self.try_load_texture(render_core, id, texture_path) {
                        panic!(e);
                    }
                }

                fn try_load_texture(&mut self, render_core: &RenderCore, id: &str, texture_path: &str) -> Result<(), String> {
                    let texture = self.manager.files.remove(texture_path);
                    if texture.is_none() {
                        return Err(format!("Texture file {} not found!", texture_path));
                    }
                    let texture = texture.unwrap();
                    let texture = render_core.create_texture(texture.contents().into_vec());
                    self.manager.textures.insert(id.to_string(), texture);
                    Ok(())
                }

                fn prepare_texture(&mut self, tex_id: &str, id: &str) {
                    if let Err(e) = self.try_prepare_texture(tex_id, id) {
                        panic!(e);
                    }
                }

                fn try_prepare_texture(&mut self, tex_id: &str, id: &str) -> Result<(), String> {
                    let texture = self.manager.textures.get(tex_id);
                    if texture.is_none() {
                        return Err(format!("Texture {} not found!", tex_id));
                    }
                    self.manager.texture_regions.insert(id.to_string(), texture.unwrap().to_region());
                    Ok(())
                }

                fn crop_texture_region(&mut self, tex_id: &str, id: &str, x: u16, y: u16, width: u16, height: u16) {
                    if let Err(e) = self.try_crop_texture_region(tex_id, id, x, y, width, height) {
                        panic!(e);
                    }
                }

                fn try_crop_texture_region(&mut self, tex_id: &str, id: &str, x: u16, y: u16, width: u16, height: u16) -> Result<(), String> {
                    let texture = self.manager.textures.get(tex_id);
                    if texture.is_none() {
                        return Err(format!("Texture {} not found!", tex_id));
                    }
                    self.manager.texture_regions.insert(id.to_string(), texture.unwrap().crop(x, y, width, height));
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