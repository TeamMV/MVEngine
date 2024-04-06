use std::any::Any;
use crate::render::texture::Texture;

pub use shaderc::ShaderKind;
use crate::asset::manager::AssetHandle;
use crate::render::backend::shader::Shader;
use crate::render::model::Model;

pub struct AssetData {
    pub(crate) ty: AssetType,
    pub(crate) path: String
}

pub enum AssetType {
    Texture,
    Model,
    Shader(ShaderKind)
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
    pub(crate) data: AssetData,
    pub(crate) handle: AssetHandle,
}

impl Asset {
    pub(crate) fn load(&mut self) {

    }

    pub(crate) fn unload(&mut self) {

    }

    pub(crate) fn reload(&mut self) {
        // This isn't the same as calling unload then load,
        // since we want to load it into memory, swap it with the loaded asset,
        // then unloaded the memory from the previous asset
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

    pub fn take_error(&self) -> Option<Box<dyn Any + Send + 'static>> {
        if self.failed() {
            let unsafe_ref = unsafe { &mut *(&self.inner as *const _ as *mut InnerAsset) };
            let InnerAsset::Failed(err) = std::mem::replace(unsafe_ref, InnerAsset::Unloaded) else { unreachable!() };
            Some(err)
        } else {
            None
        }
    }
}