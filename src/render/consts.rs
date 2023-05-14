use std::collections::HashMap;
use std::num::{NonZeroU32, NonZeroU64};
use std::sync::Arc;
use once_cell::unsync::Lazy;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BufferBinding, BufferBindingType, BufferSize, Sampler, SamplerBindingType, ShaderStages, TextureSampleType, TextureView, TextureViewDimension, vertex_attr_array, VertexBufferLayout, VertexStepMode};
use crate::render::common::ShaderType::Fragment;
use crate::render::common::Texture;

pub(crate) const VERT_LIMIT: u64 = 10000;
pub(crate) const VERT_LIMIT_2D_FLOATS: u64 = VERT_LIMIT * VERTEX_LAYOUT_2D.array_stride / 4;
pub(crate) const VERT_LIMIT_2D_BYTES: u64 = VERT_LIMIT * VERTEX_LAYOUT_2D.array_stride;
pub(crate) const VERT_LIMIT_MODEL_3D_BYTES: u64 = VERT_LIMIT * VERTEX_LAYOUT_MODEL_3D.array_stride;
pub(crate) const VERT_LIMIT_BATCH_3D_BYTES: u64 = VERT_LIMIT * VERTEX_LAYOUT_BATCH_3D.array_stride;
pub(crate) const INDEX_LIMIT: u64 = VERT_LIMIT * 6;

pub(crate) const EFFECT_VERT: &str = "#version 450\nlayout(location=0)out vec2 fTexCoord;vec2 positions[4]=vec2[](vec2(-1.0,-1.0),vec2(-1.0,1.0),vec2(1.0,-1.0),vec2(1.0,1.0));vec2 tex[4]=vec2[](vec2(0.0,1.0),vec2(0.0,0.0),vec2(1.0,1.0),vec2(1.0,0.0));void main(){fTexCoord=tex[gl_VertexIndex];gl_Position=vec4(positions[gl_VertexIndex],0.0,1.0);}";

pub(crate) const EFFECT_INDICES: [u32; 6] = [0, 2, 1, 1, 2, 3];

pub(crate) static mut DEFAULT_SAMPLER: Option<Sampler> = None;
pub(crate) static mut DUMMY_TEXTURE: Option<Arc<Texture>> = None;

pub(crate) static mut MAX_TEXTURES: usize = 0;
pub(crate) const TEXTURE_LIMIT: usize = 255;

pub(crate) const VERTEX_LAYOUT_2D: VertexBufferLayout = VertexBufferLayout {
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

pub(crate) const VERTEX_LAYOUT_MODEL_3D: VertexBufferLayout = VertexBufferLayout {
    array_stride: 36,
    step_mode: VertexStepMode::Vertex,
    attributes: &vertex_attr_array![
        0 => Float32x3, //pos
        1 => Float32x3, //normal
        2 => Float32x2, //uv
        3 => Float32    //material id
    ]
};

pub(crate) const VERTEX_LAYOUT_BATCH_3D: VertexBufferLayout = VertexBufferLayout {
    array_stride: 64,
    step_mode: VertexStepMode::Vertex,
    attributes: &vertex_attr_array![
        0 => Float32x3, //pos
        1 => Float32x3, //normal
        2 => Float32x2, //uv
        3 => Float32,   //material id
        4 => Float32x4, //canvas coords
        5 => Float32x2, //canvas data
        6 => Float32    //model matrix idx
    ]
};

pub(crate) const VERTEX_LAYOUT_NONE: VertexBufferLayout = VertexBufferLayout {
    array_stride: 0,
    step_mode: VertexStepMode::Vertex,
    attributes: &vertex_attr_array![]
};

pub(crate) const BIND_GROUP_2D: u8 = 0;
pub(crate) const BIND_GROUP_TEXTURES_2D: u8 = 1;
pub(crate) const BIND_GROUP_GEOMETRY_MODEL_3D: u8 = 2;
pub(crate) const BIND_GROUP_GEOMETRY_BATCH_3D: u8 = 3;
pub(crate) const BIND_GROUP_LIGHTING_3D: u8 = 4;
pub(crate) const BIND_GROUP_MODEL_3D: u8 = 5;
pub(crate) const BIND_GROUP_BATCH_3D: u8 = 6;
pub(crate) const BIND_GROUP_EFFECT: u8 = 7;
pub(crate) const BIND_GROUP_EFFECT_CUSTOM: u8 = 8;

pub(crate) static mut BIND_GROUPS: Lazy<HashMap<u8, BindGroupLayout>> = Lazy::new(HashMap::new);

pub(crate) const BIND_GROUP_LAYOUT_2D: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("Bind group layout 2D"),
    entries: &[
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(128) }),
            } ,
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        }
    ],
};

pub(crate) const BIND_GROUP_LAYOUT_GEOMETRY_MODEL_3D: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("Bind group layout geometry pass (model) 3D"),
    entries: &[],
};

pub(crate) const BIND_GROUP_LAYOUT_GEOMETRY_BATCH_3D: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("Bind group layout geometry pass (batch) 3D"),
    entries: &[],
};

pub(crate) const BIND_GROUP_LAYOUT_LIGHTING_3D: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("Bind group layout lighting pass (model) 3D"),
    entries: &[],
};

pub(crate) const BIND_GROUP_LAYOUT_MODEL_3D: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("Bind group layout forward render (model) 3D"),
    entries: &[],
};

pub(crate) const BIND_GROUP_LAYOUT_BATCH_3D: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("Bind group layout forward render (batch) 3D"),
    entries: &[],
};

pub(crate) const BIND_GROUP_LAYOUT_EFFECT: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("Bind group layout effect"),
    entries: &[
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                multisampled: false,
                view_dimension: TextureViewDimension::D2,
                sample_type: TextureSampleType::Float { filterable: true },
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(16) }),
            },
            count: None
        }
    ],
};

pub(crate) const BIND_GROUP_LAYOUT_EFFECT_CUSTOM: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("Bind group layout effect custom"),
    entries: &[
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        }
    ],
};