use crate::asset::manager::AssetHandle;
use crate::math::vec::Vec4;

pub struct Model {

}

pub struct Material {
    color: Vec4,
    color_tex: Option<AssetHandle>,
    metallic: f32,
    metallic_tex: Option<AssetHandle>,
    roughness: f32,
    roughness_tex: Option<AssetHandle>,
    reflectance: f32,
    reflectance_tex: Option<AssetHandle>,
    clear_coat: f32,
    clear_coat_tex: Option<AssetHandle>,
    clear_coat_roughness: f32,
    clear_coat_roughness_tex: Option<AssetHandle>,
    anisotropy: f32,
    anisotropy_tex: Option<AssetHandle>,
    anisotropy_direction: Vec4,
    ambient_occlusion: f32,
    ambient_occlusion_tex: Option<AssetHandle>,
    normal: Vec4,
    normal_tex: Option<AssetHandle>,
    clear_coat_normal: Vec4,
    clear_coat_normal_tex: Option<AssetHandle>,
    emissive: Vec4,
    emissive_tex: Option<AssetHandle>,
    ior: f32,
    transmission: f32,
    absorption: f32,
    thickness: f32,
    sheen_color: Vec4,
}
