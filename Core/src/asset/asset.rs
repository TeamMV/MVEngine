use crate::render::texture::Texture;

pub struct AssetData {
    pub(crate) ty: AssetType,
    pub(crate) path: String
}

pub enum AssetType {
    Texture,
    Model
}

pub enum Asset {
    Texture(Texture),
    Model(),
    Unloaded(AssetData),
}

impl Asset {
    pub(crate) fn load(&mut self) {

    }

    pub(crate) fn unload(&mut self) {

    }
}