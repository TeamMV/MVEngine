#![allow(non_snake_case)]

use mvengine_ui_parsing::json::from_json::FromJsonTrait;
use mvengine_ui_parsing::json::types::JsonElement;
use mvengine_ui_parsing::json::from_json::FromJsonError;
use mvengine_proc_macro::FromJson;

#[derive(FromJson)]
pub struct GLTFFile {
    pub asset: GLTFAsset,
    #[default_value(0)]
    pub scene: u32,
    pub scenes: Vec<GLTFScene>,
    pub accessors: Vec<GLTFAccessor>,
    pub bufferViews: Vec<GLTFBufferView>,
    pub buffers: Vec<GLTFBuffer>,
}

#[derive(FromJson)]
pub struct GLTFAsset {
    #[default_value(String::new())]
    pub generator: String,
    pub version: String,
}

#[derive(FromJson)]
pub struct GLTFScene {
    #[default_value(String::new())]
    pub name: String,
    #[default_value(Vec::new())]
    pub nodes: Vec<u32>,
}

#[derive(FromJson)]
pub struct GLTFAccessor {
    #[default_value(0)]
    pub bufferView: u32,
    #[default_value(0)]
    pub byteOffset: u32,
    pub componentType: GLTFComponentType,
    pub count: u32,

    #[jsonkey("type")]
    pub type_: GLTFAccessorType,

    #[default_value(Vec::new())]
    pub max: Vec<f32>,
    #[default_value(Vec::new())]
    pub min: Vec<f32>,
    #[default_value(false)]
    pub normalized: bool,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, FromJson)]
pub enum GLTFAccessorType {
    SCALAR,
    VEC2,
    VEC3,
    VEC4,
    MAT2,
    MAT3,
    MAT4,
}

#[derive(FromJson)]
pub struct GLTFBufferView {
    pub buffer: u32,
    #[default_value(0)]
    pub byteOffset: u32,
    #[default_value(0)]
    pub byteStride: u32,
    pub byteLength: u32,
    #[default_value(0)]
    pub target: u32,
}

#[derive(FromJson)]
pub struct GLTFBuffer {
    pub uri: Option<String>,
    pub byteLength: u32,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum GLTFComponentType {
    I8,
    U8,
    I16,
    U16,
    U32,
    F32,
}

impl FromJsonTrait for GLTFComponentType {
    fn from_json(json: &JsonElement) -> Result<Self, FromJsonError>
    where
        Self: Sized
    {
        let component_type = Self::illegal_conversion(json.as_int())?;

        let variant = match component_type {
            5120 => GLTFComponentType::I8,   // BYTE
            5121 => GLTFComponentType::U8,   // UNSIGNED_BYTE
            5122 => GLTFComponentType::I16,  // SHORT
            5123 => GLTFComponentType::U16,  // UNSIGNED_SHORT
            5125 => GLTFComponentType::U32,  // UNSIGNED_INT
            5126 => GLTFComponentType::F32,  // FLOAT
            other => return Err(FromJsonError::NoSuchField(
                format!("Invalid GLTF component type: {other}")
            )),
        };
        Ok(variant)
    }
}