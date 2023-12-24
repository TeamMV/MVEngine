use std::collections::HashMap;
use std::num::NonZeroU64;
use std::sync::Arc;

use mvutils::once::{CreateOnce, LazyInitOnce};
use mvutils::{create_once, lazy_init};
use wgpu::{
    vertex_attr_array, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BufferBindingType, Sampler, SamplerBindingType, ShaderStages, TextureSampleType,
    TextureViewDimension, VertexBufferLayout, VertexStepMode,
};

use crate::render::common::Texture;

pub(crate) const VERT_LIMIT: u64 = 10000;
pub(crate) const VERT_LIMIT_2D_FLOATS: u64 = VERT_LIMIT * VERTEX_LAYOUT_2D.array_stride / 4;
pub(crate) const VERT_LIMIT_2D_BYTES: u64 = VERT_LIMIT * VERTEX_LAYOUT_2D.array_stride;
pub(crate) const VERT_LIMIT_MODEL_3D_BYTES: u64 = VERT_LIMIT * VERTEX_LAYOUT_MODEL_3D.array_stride;
pub(crate) const VERT_LIMIT_BATCH_3D_BYTES: u64 = VERT_LIMIT * VERTEX_LAYOUT_BATCH_3D.array_stride;
pub(crate) const INDEX_LIMIT: u64 = VERT_LIMIT * 6;

pub(crate) const VERTEX_2D_SIZE_FLOATS: usize = 22;
pub(crate) const VERTEX_3D_MODEL_SIZE_FLOATS: usize = 9;
pub(crate) const VERTEX_3D_BATCH_SIZE_FLOATS: usize = 16;

pub(crate) const DUMMY_VERT: &str = "#version 450\nlayout(location=0)out vec2 fTexCoord;vec2 p[4]=vec2[](vec2(-1.0,-1.0),vec2(-1.0,1.0),vec2(1.0,-1.0),vec2(1.0,1.0));vec2 t[4]=vec2[](vec2(0.0,1.0),vec2(0.0,0.0),vec2(1.0,1.0),vec2(1.0,0.0));void main(){fTexCoord=t[gl_VertexIndex];gl_Position=vec4(p[gl_VertexIndex],0.0,1.0);}";

pub(crate) const EFFECT_INDICES: [u32; 6] = [0, 2, 1, 1, 2, 3];

create_once! {
    pub(crate) static DEFAULT_SAMPLER: Sampler;
    pub(crate) static DUMMY_TEXTURE: Arc<Texture>;
    pub(crate) static MAX_TEXTURES: usize;
    pub(crate) static MAX_LIGHTS: usize;
}

pub(crate) const TEXTURE_LIMIT: usize = 255;
pub(crate) const LIGHT_LIMIT: usize = 255;

pub(crate) const VERTEX_LAYOUT_2D: VertexBufferLayout = VertexBufferLayout {
    array_stride: 88,
    step_mode: VertexStepMode::Vertex,
    attributes: &vertex_attr_array![
        0 => Float32x3,  //pos
        1 => Float32,    //rot
        2 => Float32x2,  //rot origin
        3 => Float32x4,  //color
        4 => Float32x2,  //uv
        5 => Float32,    //tex id
        6 => Float32,    //use cam
        7 => Float32,    //trans rotation
        8 => Float32x2,  //trans translation
        9 => Float32x2,  //trans origin
        10 => Float32x2, //trans scale
        11 => Float32,   //is font
    ],
};

pub(crate) const VERTEX_LAYOUT_MODEL_3D: VertexBufferLayout = VertexBufferLayout {
    array_stride: 36,
    step_mode: VertexStepMode::Vertex,
    attributes: &vertex_attr_array![
        0 => Float32x3, //pos
        1 => Float32x3, //normal
        2 => Float32x2, //uv
        3 => Float32    //material id
    ],
};

pub(crate) const VERTEX_LAYOUT_MODEL_3D_MAT_ID_OFFSET: usize = 8;

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
    ],
};

pub(crate) const VERTEX_LAYOUT_NONE: VertexBufferLayout = VertexBufferLayout {
    array_stride: 0,
    step_mode: VertexStepMode::Vertex,
    attributes: &vertex_attr_array![],
};

pub(crate) const BIND_GROUP_2D: u8 = 0;
pub(crate) const BIND_GROUP_TEXTURES: u8 = 1;
pub(crate) const BIND_GROUP_MODEL_MATRIX: u8 = 2;
pub(crate) const BIND_GROUP_GEOMETRY_MODEL_3D: u8 = 3;
pub(crate) const BIND_GROUP_GEOMETRY_BATCH_3D: u8 = 4;
pub(crate) const BIND_GROUP_LIGHTING_3D: u8 = 5;
pub(crate) const BIND_GROUP_MODEL_3D: u8 = 6;
pub(crate) const BIND_GROUP_BATCH_3D: u8 = 7;
pub(crate) const BIND_GROUP_EFFECT: u8 = 8;
pub(crate) const BIND_GROUP_EFFECT_CUSTOM: u8 = 9;

lazy_init! {
    pub(crate) static BIND_GROUPS: HashMap<u8, BindGroupLayout> = HashMap::new();
}

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
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(4) }),
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        },
    ],
};

pub(crate) const BIND_GROUP_LAYOUT_MODEL_MATRIX: BindGroupLayoutDescriptor =
    BindGroupLayoutDescriptor {
        label: Some("Bind group layout model matrix"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    };

pub(crate) const BIND_GROUP_LAYOUT_GEOMETRY_MODEL_3D: BindGroupLayoutDescriptor =
    BindGroupLayoutDescriptor {
        label: Some("Bind group layout geometry pass (model) 3D"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(unsafe { NonZeroU64::new_unchecked(128) }),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ],
    };

pub(crate) const BIND_GROUP_LAYOUT_GEOMETRY_BATCH_3D: BindGroupLayoutDescriptor =
    BindGroupLayoutDescriptor {
        label: Some("Bind group layout geometry pass (batch) 3D"),
        entries: &[],
    };

pub(crate) const BIND_GROUP_LAYOUT_LIGHTING_3D: BindGroupLayoutDescriptor =
    BindGroupLayoutDescriptor {
        label: Some("Bind group layout lighting pass (model) 3D"),
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
                ty: BindingType::Texture {
                    multisampled: false,
                    view_dimension: TextureViewDimension::D2,
                    sample_type: TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    multisampled: false,
                    view_dimension: TextureViewDimension::D2,
                    sample_type: TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 4,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    };

pub(crate) const BIND_GROUP_LAYOUT_MODEL_3D: BindGroupLayoutDescriptor =
    BindGroupLayoutDescriptor {
        label: Some("Bind group layout forward render (model) 3D"),
        entries: &[],
    };

pub(crate) const BIND_GROUP_LAYOUT_BATCH_3D: BindGroupLayoutDescriptor =
    BindGroupLayoutDescriptor {
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
            count: None,
        },
    ],
};

pub(crate) const BIND_GROUP_LAYOUT_EFFECT_CUSTOM: BindGroupLayoutDescriptor =
    BindGroupLayoutDescriptor {
        label: Some("Bind group layout effect custom"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    };
