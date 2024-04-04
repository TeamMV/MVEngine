use crate::render::texture::Texture;

pub struct AssetData {
    ty: AssetType,
    path: String
}

enum AssetType {
    Texture,
    Model
}

pub enum Asset {
    Texture(Texture),
    Model(),
    Unloaded(AssetData),
}