use std::cmp::Ordering;
use glsl_to_spirv::{compile, ShaderType};
use wgpu::{AddressMode, Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout, Origin3d, RenderPipeline, Sampler, SamplerDescriptor, ShaderModuleDescriptor, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, VertexBufferLayout};
use wgpu::util::make_spirv;
use crate::render::consts::{EFFECT_VERT, VERTEX_LAYOUT_EFFECT};
use crate::render::init::{PipelineBuilder, State};

use std::io::Read;
use std::num::NonZeroU32;
use glam::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};
use image::GenericImageView;
use mvutils::utils::next_id;
use regex::internal::Input;

macro_rules! compile_spv {
    ($code:expr, $t:expr) => {
        compile($code, $t).map_err(|e| panic!("{}", e)).unwrap().bytes().flatten().collect::<Vec<_>>()
    };
}

macro_rules! epecpc {
    ($($t:ty),*) => {
        $(
            impl PartialEq<Self> for $t {
                fn eq(&self, other: &Self) -> bool {
                    self.id == other.id
                }
            }

            impl Eq for $t {}

            impl PartialOrd<Self> for $t {
                fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                    Some(self.id.cmp(&other.id))
                }
            }

            impl Ord for $t {
                fn cmp(&self, other: &Self) -> Ordering {
                    self.id.cmp(&other.id)
                }
            }
        )*
    };
}

pub struct Shader {
    id: u64,
    vert: Option<Vec<u8>>,
    frag: Option<Vec<u8>>,
    pipeline: Option<RenderPipeline>,
    stripped_pipeline: Option<RenderPipeline>,
}

impl Clone for Shader {
    fn clone(&self) -> Self {
        if self.pipeline.is_some() || self.stripped_pipeline.is_some() {
            panic!("Cannot clone a shader that is already made!");
        }
        Shader {
            id: self.id,
            vert: self.vert.clone(),
            frag: self.frag.clone(),
            pipeline: None,
            stripped_pipeline: None,
        }
    }
}

impl Shader {
    pub(crate) fn compile_glsl(code: String, shader_type: ShaderType) -> Vec<u8> {
        compile_spv!(code.as_str(), shader_type)
    }

    pub(crate) fn new(vert: Vec<u8>, frag: Vec<u8>) -> Self {
        Self {
            id: next_id("MVCore::Shader"),
            vert: Some(vert),
            frag: Some(frag),
            pipeline: None,
            stripped_pipeline: None,
        }
    }

    pub(crate) fn new_glsl(vert: String, frag: String) -> Self {
        let v_spv = compile_spv!(vert.as_str(), ShaderType::Vertex);
        let f_spv = compile_spv!(frag.as_str(), ShaderType::Fragment);
        Self {
            id: next_id("MVCore::Shader"),
            vert: Some(v_spv),
            frag: Some(f_spv),
            pipeline: None,
            stripped_pipeline: None,
        }
    }

    pub(crate) fn setup_pipeline(mut self, state: &State, layout: VertexBufferLayout<'static>, bind_groups:  &[u8]) -> Self {
        if self.vert.is_none() || self.frag.is_none() {
            return self;
        }

        let vert = self.vert.take().unwrap();

        let vert = state.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("vert"),
            source: make_spirv(&vert),
        });

        let frag = self.frag.take().unwrap();

        let frag = state.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("frag"),
            source: make_spirv(&frag),
        });

        self.pipeline = Some(
            PipelineBuilder::begin(state)
                .custom_vertex_layout(layout.clone())
                .param(PipelineBuilder::RENDER_MODE, PipelineBuilder::RENDER_MODE_TRIANGLES)
                .shader(PipelineBuilder::SHADER_VERTEX, &vert)
                .shader(PipelineBuilder::SHADER_FRAGMENT, &frag)
                .bind_groups(bind_groups)
                .build()
        );

        self.stripped_pipeline = Some(
            PipelineBuilder::begin(state)
                .custom_vertex_layout(layout)
                .param(PipelineBuilder::RENDER_MODE, PipelineBuilder::RENDER_MODE_TRIANGLE_STRIP)
                .shader(PipelineBuilder::SHADER_VERTEX, &vert)
                .shader(PipelineBuilder::SHADER_FRAGMENT, &frag)
                .bind_groups(bind_groups)
                .build()
        );
        self
    }

    pub(crate) fn get_pipeline(&self) -> &RenderPipeline {
        self.pipeline.as_ref().expect("Binding unmade shader!")
    }

    pub(crate) fn get_stripped_pipeline(&self) -> &RenderPipeline {
        self.stripped_pipeline.as_ref().expect("Binding unmade shader!")
    }
}

pub struct EffectShader {
    id: u64,
    shader: Option<Vec<u8>>,
    pipeline: Option<RenderPipeline>
}

impl Clone for EffectShader {
    fn clone(&self) -> Self {
        if self.pipeline.is_some() {
            panic!("Cannot clone a shader that is already made!");
        }
        EffectShader {
            id: self.id,
            shader: self.shader.clone(),
            pipeline: None,
        }
    }
}

impl EffectShader {
    pub(crate) fn new(shader: Vec<u8>) -> Self {
        Self {
            id: next_id("MVCore::EffectShader"),
            shader: Some(shader),
            pipeline: None,
        }
    }

    pub(crate) fn new_glsl(shader: &str) -> Self {
        let spv = compile_spv!(shader, ShaderType::Fragment);
        Self {
            id: next_id("MVCore::EffectShader"),
            shader: Some(spv),
            pipeline: None,
        }
    }

    pub(crate) fn setup_pipeline(mut self, state: &State, bind_groups: &[u8]) -> Self {
        if self.shader.is_none() {
            return self;
        }

        let vert = compile_spv!(EFFECT_VERT, ShaderType::Vertex);

        let vert = state.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("effect_vert"),
            source: make_spirv(&vert),
        });

        let frag = self.shader.take().unwrap();

        let frag = state.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("effect_frag"),
            source: make_spirv(&frag),
        });

        self.pipeline = Some(
            PipelineBuilder::begin(state)
                .custom_vertex_layout(VERTEX_LAYOUT_EFFECT)
                .param(PipelineBuilder::RENDER_MODE, PipelineBuilder::RENDER_MODE_TRIANGLES)
                .shader(PipelineBuilder::SHADER_VERTEX, &vert)
                .shader(PipelineBuilder::SHADER_FRAGMENT, &frag)
                .bind_groups(bind_groups)
                .build()
        );

        self
    }

    pub(crate) fn get_pipeline(&self) -> &RenderPipeline {
        self.pipeline.as_ref().expect("Binding unmade shader!")
    }
}

pub struct Texture {
    id: u64,
    image: Option<Vec<u8>>,
    texture: Option<wgpu::Texture>,
    view: Option<TextureView>
}

impl Texture {
    pub(crate) fn new(image: Vec<u8>) -> Self {
        Self {
            id: next_id("MVCore::Texture"),
            image: Some(image),
            texture: None,
            view: None
        }
    }

    pub(crate) fn premade(texture: wgpu::Texture, view: TextureView) -> Self {
        Self {
            id: next_id("MVCore::Texture"),
            image: None,
            texture: Some(texture),
            view: Some(view)
        }
    }

    pub(crate) fn make(&mut self, state: &State) {
        if self.image.is_none() {
            return;
        }

        let image = self.image.take().unwrap();
        let image = image::load_from_memory(&image).unwrap();
        let bytes = image.to_rgba8();
        let dimensions = image.dimensions();

        let size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = state.device.create_texture(&TextureDescriptor {
            label: Some("Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        state.queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            // The actual pixel data
            &bytes,
            // The layout of the texture
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size
        );

        let view = texture.create_view(&TextureViewDescriptor::default());

        self.texture = Some(texture);
        self.view = Some(view);
    }

    pub(crate) fn get_texture(&self) -> &wgpu::Texture {
        self.texture.as_ref().expect("Binding unmade texture!")
    }

    pub(crate) fn get_view(&self) -> &TextureView {
        self.view.as_ref().expect("Binding unmade texture!")
    }
}

epecpc!(Shader, EffectShader, Texture);

pub trait Bytes {
    fn cast_bytes(&self) -> &[u8];
}

impl Bytes for &[f32] {
    fn cast_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.as_ptr() as *const u8, self.len() * 4) }
    }
}

impl Bytes for &[u32] {
    fn cast_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.as_ptr() as *const u8, self.len() * 4) }
    }
}

macro_rules! impl_bytes_glam {
    ($($t:ty = $s:literal),*) => {
        $(
            impl Bytes for $t {
                fn cast_bytes(&self) -> &[u8] {
                    unsafe { core::slice::from_raw_parts(self.as_ref().as_ptr() as *const u8, $s) }
                }
            }
        )*
    };
}

impl_bytes_glam!(Vec2 = 8, Vec3 = 12, Vec4 = 16, Mat2 = 16, Mat3 = 36, Mat4 = 64);