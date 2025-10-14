#![allow(non_snake_case)]

use mvengine_ui_parsing::json::from_json::FromJsonTrait;
use mvengine_ui_parsing::json::types::JsonElement;
use mvengine_ui_parsing::json::from_json::FromJsonError;
use mvengine_proc_macro::FromJson;

#[derive(FromJson)]
pub struct GLTFFile {
    pub asset: GLTFAsset,
    pub scene: Option<u32>,
    pub scenes: Option<Vec<GLTFScene>>,
    pub nodes: Option<Vec<GLTFNode>>,
    pub meshes: Option<Vec<GLTFMesh>>,
    pub materials: Option<Vec<GLTFMaterial>>,
    pub buffers: Option<Vec<GLTFBuffer>>,
    pub bufferViews: Option<Vec<GLTFBufferView>>,
    pub accessors: Option<Vec<GLTFAccessor>>,
    pub images: Option<Vec<GLTFImage>>,
    pub textures: Option<Vec<GLTFTexture>>,
}

#[derive(FromJson)]
pub struct GLTFAsset {
    pub generator: Option<String>,
    pub version: String,
    pub copyright: Option<String>,
}

#[derive(FromJson)]
pub struct GLTFScene {
    pub name: Option<String>,
    pub nodes: Option<Vec<u32>>,
}

#[derive(FromJson)]
pub struct GLTFNode {
    pub name: Option<String>,
    pub mesh: Option<u32>,
    pub children: Option<Vec<u32>>,
    pub translation: Option<[f32; 3]>,
    pub rotation: Option<[f32; 4]>,
    pub scale: Option<[f32; 3]>,
    pub matrix: Option<[f32; 16]>,
}

#[derive(FromJson)]
pub struct GLTFMesh {
    pub name: Option<String>,
    pub primitives: Vec<GLTFPrimitive>,
    pub weights: Option<Vec<f32>>,
}

#[derive(FromJson)]
pub struct GLTFPrimitive {
    pub attributes: GLTFAttributes,
    pub indices: Option<u32>,
    pub material: Option<u32>,
    #[default_value(4)]
    pub mode: u32, // defaults to 4 (TRIANGLES)
}

#[derive(FromJson)]
pub struct GLTFAttributes {
    pub POSITION: Option<u32>,
    pub NORMAL: Option<u32>,
    pub TEXCOORD_0: Option<u32>,
    pub TANGENT: Option<u32>,
}

#[derive(FromJson)]
pub struct GLTFMaterial {
    pub name: Option<String>,
    pub pbrMetallicRoughness: Option<GLTFPBRMetallicRoughness>,
    pub normalTexture: Option<GLTFTextureInfo>,
    pub occlusionTexture: Option<GLTFTextureInfo>,
    pub emissiveTexture: Option<GLTFTextureInfo>,
    pub emissiveFactor: Option<[f32; 3]>,
    pub alphaMode: Option<String>,
    pub alphaCutoff: Option<f32>,
    pub doubleSided: Option<bool>,
}

#[derive(FromJson)]
pub struct GLTFPBRMetallicRoughness {
    pub baseColorFactor: Option<[f32; 4]>,
    pub metallicFactor: Option<f32>,
    pub roughnessFactor: Option<f32>,
    pub baseColorTexture: Option<GLTFTextureInfo>,
    pub metallicRoughnessTexture: Option<GLTFTextureInfo>,
}

#[derive(FromJson)]
pub struct GLTFTextureInfo {
    pub index: u32,
    pub texCoord: Option<u32>,
}

#[derive(FromJson)]
pub struct GLTFImage {
    pub uri: Option<String>,
    pub mimeType: Option<String>,
    pub bufferView: Option<u32>,
    pub name: Option<String>,
}

#[derive(FromJson)]
pub struct GLTFTexture {
    pub sampler: Option<u32>,
    pub source: Option<u32>,
}

#[derive(FromJson)]
pub struct GLTFBuffer {
    pub uri: Option<String>,
    pub byteLength: u32,
}

#[derive(FromJson)]
pub struct GLTFBufferView {
    pub buffer: u32,
    pub byteOffset: Option<u32>,
    pub byteLength: u32,
    pub byteStride: Option<u32>,
    pub target: Option<u32>,
}

#[derive(FromJson)]
pub struct GLTFAccessor {
    pub bufferView: Option<u32>,
    pub byteOffset: Option<u32>,
    pub componentType: u32,
    pub count: u32,
    pub type_: String, // “VEC3”, “SCALAR”, “MAT4”, etc.
    pub max: Option<Vec<f32>>,
    pub min: Option<Vec<f32>>,
    pub normalized: Option<bool>,
}

//TODO FromJson for enumns and have #[jsonkey("mykey)] for all fields in both struct and enum
#[derive(FromJson)]
pub enum GLTFAccessorType {
    VEC3
}