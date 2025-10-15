use crate::rendering::loading::gltf::v2::json::{GLTFAccessor, GLTFAccessorType, GLTFComponentType};

pub struct Buffer<'a> {
    slice: &'a [u8]
}

pub struct BufferView<'a> {
    buffer: Buffer<'a>,
    offset: usize,
    length: usize,
    stride: Option<usize>,
}

pub struct Accessor<'a> {
    view: BufferView<'a>,
    component_type: GLTFComponentType,
    count: usize,
    offset: usize,
    type_: GLTFAccessorType,
}

impl<'a> Accessor<'a> {
    pub fn from_json_representation(view: BufferView<'a>, json: &GLTFAccessor) -> Self {
        Self {
            view,
            component_type: json.componentType,
            count: json.count as usize,
            offset: json.byteOffset as usize,
            type_: json.type_,
        }
    }
}