use crate::render::texture::Texture;
use std::any::Any;

use crate::asset::manager::AssetHandle;
use crate::render::backend::shader::Shader;
use crate::render::model::Model;
pub use shaderc::ShaderKind;

pub enum AssetType {
    Texture,
    Model,
    Shader(ShaderKind),
}

pub enum InnerAsset {
    Texture(Texture),
    Model(Model),
    Shader(Shader),
    Unloaded,
    Failed(Box<dyn Any + Send + 'static>),
}

pub struct Asset {
    pub(crate) inner: InnerAsset,
    pub(crate) ty: AssetType,
    pub(crate) handle: AssetHandle,
}

impl Asset {
    pub(crate) fn load(&mut self) {
        std::thread::sleep(std::time::Duration::from_secs(3));
        if self.is_loaded() { return; }
        let loader = self.handle.get_manager().get_loader();
        match self.ty {
            AssetType::Texture => {
                match loader.import_texture(self.handle.clone()) {
                    Ok(texture) => self.inner = InnerAsset::Texture(texture),
                    Err(err) => self.inner = InnerAsset::Failed(Box::new(err)),
                }
            }
            AssetType::Model => {}
            AssetType::Shader(kind) => {}
        }
    }

    pub(crate) fn unload(&mut self) {
        if !self.is_loaded() {
            return;
        }
    }

    pub(crate) fn reload(&mut self) {
        // This isn't the same as calling unload then load,
        // since we want to load it into memory, swap it with the loaded asset,
        // then unloaded the memory from the previous asset
    }

    pub fn handle(&self) -> AssetHandle {
        self.handle.clone()
    }

    pub fn is_loaded(&self) -> bool {
        !matches!(self.inner, InnerAsset::Failed(_) | InnerAsset::Unloaded)
    }

    pub fn failed(&self) -> bool {
        matches!(self.inner, InnerAsset::Failed(_))
    }

    pub fn error(&self) -> Option<&Box<dyn Any + Send + 'static>> {
        match &self.inner {
            InnerAsset::Failed(err) => Some(err),
            _ => None,
        }
    }

    #[allow(invalid_reference_casting)]
    pub fn take_error(&self) -> Option<Box<dyn Any + Send + 'static>> {
        if self.failed() {
            let unsafe_ref = unsafe { &mut *(&self.inner as *const _ as *mut InnerAsset) };
            let InnerAsset::Failed(err) = std::mem::replace(unsafe_ref, InnerAsset::Unloaded)
            else {
                unreachable!()
            };
            Some(err)
        } else {
            None
        }
    }

    pub fn as_texture(&self) -> Option<Texture> {
        match &self.inner {
            InnerAsset::Texture(texture) => Some(texture.clone()),
            _ => None,
        }
    }
}
