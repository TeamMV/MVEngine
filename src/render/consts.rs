use std::collections::HashMap;
use std::num::{NonZeroU32, NonZeroU64};
use once_cell::unsync::Lazy;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BufferBinding, BufferBindingType, BufferSize, Sampler, SamplerBindingType, ShaderStages, TextureSampleType, TextureView, TextureViewDimension, vertex_attr_array, VertexBufferLayout, VertexStepMode};
use crate::render::common::Texture;

pub const VERT_LIMIT: u64 = 10000;
pub const VERT_LIMIT_2D_BYTES: u64 = VERT_LIMIT * VERTEX_LAYOUT_2D.array_stride;
pub const VERT_LIMIT_MODEL_3D_BYTES: u64 = VERT_LIMIT * VERTEX_LAYOUT_MODEL_3D.array_stride;
pub const VERT_LIMIT_BATCH_3D_BYTES: u64 = VERT_LIMIT * VERTEX_LAYOUT_BATCH_3D.array_stride;
pub const INDEX_LIMIT: u64 = VERT_LIMIT * 6;

pub const EFFECT_VERT: &str = "#version 450\nlayout(location=0)out vec2 fTexCoord;vec2 positions[4]=vec2[](vec2(-1.0,-1.0),vec2(-1.0,1.0),vec2(1.0,-1.0),vec2(1.0,1.0));vec2 tex[4]=vec2[](vec2(0.0,0.0),vec2(0.0,1.0),vec2(1.0,0.0),vec2(1.0,1.0));void main(){fTexCoord=tex[gl_VertexIndex];gl_Position=vec4(positions[gl_VertexIndex],0.0,1.0);}";

pub static mut DEFAULT_SAMPLER: Option<Sampler> = None;
pub static mut DUMMY_TEXTURE: Option<Texture> = None;

pub const VERTEX_LAYOUT_EFFECT: VertexBufferLayout = VertexBufferLayout {
    array_stride: 0,
    step_mode: VertexStepMode::Vertex,
    attributes: &[]
};

pub const VERTEX_LAYOUT_2D: VertexBufferLayout = VertexBufferLayout {
    array_stride: 80,
    step_mode: VertexStepMode::Vertex,
    attributes: &vertex_attr_array![
        0 => Float32x3, //pos
        1 => Float32,   //rot
        2 => Float32x2, //rot origin
        3 => Float32x4, //color
        4 => Float32x2, //uv
        5 => Float32,   //tex id
        6 => Float32x4, //canvas coords
        7 => Float32x2, //canvas data
        8 => Float32,   //use cam
    ]
};

pub const VERTEX_LAYOUT_MODEL_3D: VertexBufferLayout = VertexBufferLayout {
    array_stride: 36,
    step_mode: VertexStepMode::Vertex,
    attributes: &vertex_attr_array![
        0 => Float32x3, //pos
        1 => Float32x3, //normal
        2 => Float32x2, //uv
        3 => Float32    //material id
    ]
};

pub const VERTEX_LAYOUT_BATCH_3D: VertexBufferLayout = VertexBufferLayout {
    array_stride: 64,
    step_mode: VertexStepMode::Vertex,
    attributes: &vertex_attr_array![
        0 => Float32x3, //pos
        1 => Float32x3, //normal
        2 => Float32x2, //uv
        3 => Float32,   //material id
        4 => Float32x4, //canvas coords
        5 => Float32x2, //canvas data
        6 => Float32    //model matrix id
    ]
};

pub const BIND_GROUP_2D: u8 = 0;
pub const BIND_GROUP_TEXTURES_2D: u8 = 1;
pub const BIND_GROUP_GEOMETRY_MODEL_3D: u8 = 2;
pub const BIND_GROUP_GEOMETRY_BATCH_3D: u8 = 3;
pub const BIND_GROUP_LIGHTING_3D: u8 = 4;
pub const BIND_GROUP_MODEL_3D: u8 = 5;
pub const BIND_GROUP_BATCH_3D: u8 = 6;
pub const BIND_GROUP_EFFECT: u8 = 7;

pub static mut BIND_GROUPS: Lazy<HashMap<u8, BindGroupLayout>> = Lazy::new(HashMap::new);

pub const BIND_GROUP_LAYOUT_2D: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("Bind group layout 2D"),
    entries: &[
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    ],
};

pub const BIND_GROUP_LAYOUT_TEXTURES_2D: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("Bind group layout textures 2D"),
    entries: &[
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                multisampled: false,
                view_dimension: TextureViewDimension::D2,
                sample_type: TextureSampleType::Float { filterable: true },
            },
            count: Some(unsafe { NonZeroU32::new_unchecked(1) }),
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: Some(unsafe { NonZeroU32::new_unchecked(1) }),
        }
    ],
};

pub const BIND_GROUP_LAYOUT_GEOMETRY_MODEL_3D: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("Bind group layout geometry pass (model) 3D"),
    entries: &[],
};

pub const BIND_GROUP_LAYOUT_GEOMETRY_BATCH_3D: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("Bind group layout geometry pass (batch) 3D"),
    entries: &[],
};

pub const BIND_GROUP_LAYOUT_LIGHTING_3D: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("Bind group layout lighting pass (model) 3D"),
    entries: &[],
};

pub const BIND_GROUP_LAYOUT_MODEL_3D: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("Bind group layout forward render (model) 3D"),
    entries: &[],
};

pub const BIND_GROUP_LAYOUT_BATCH_3D: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("Bind group layout forward render (batch) 3D"),
    entries: &[],
};

pub const BIND_GROUP_LAYOUT_EFFECT: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("Bind group layout effect"),
    entries: &[],
};