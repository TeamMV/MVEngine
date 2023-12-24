use std::borrow::Cow;
use std::cmp::Ordering;
use std::mem;
use std::sync::{Arc, RwLock};

use glam::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};
use image::GenericImageView;
use mvutils::utils::{next_id, Bytecode, Recover};
use shaderc::ShaderKind;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, Extent3d, ImageCopyTexture,
    ImageDataLayout, Origin3d, RenderPipeline, ShaderModuleDescriptorSpirV, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor, VertexBufferLayout,
};

use crate::render::consts::{BIND_GROUPS, BIND_GROUP_EFFECT, BIND_GROUP_EFFECT_CUSTOM, DUMMY_VERT};
use crate::render::init::{PipelineBuilder, State};

pub enum ShaderType {
    Fragment,
    Vertex,
    Geometry,
    Compute,
    TessellationControl,
    TessellationEvaluation,
}

impl From<ShaderKind> for ShaderType {
    fn from(value: ShaderKind) -> Self {
        match value {
            ShaderKind::Fragment => ShaderType::Fragment,
            ShaderKind::Vertex => ShaderType::Vertex,
            ShaderKind::Geometry => ShaderType::Geometry,
            ShaderKind::Compute => ShaderType::Compute,
            ShaderKind::TessControl => ShaderType::TessellationControl,
            ShaderKind::TessEvaluation => ShaderType::TessellationEvaluation,
            _ => unreachable!(),
        }
    }
}

//impl Into<ShaderKind> for ShaderType {
//    fn into(self) -> ShaderKind {
//        return match self {
//            ShaderType::Fragment => ShaderKind::Fragment,
//            ShaderType::Vertex => ShaderKind::Vertex,
//            ShaderType::Geometry => ShaderKind::Geometry,
//            ShaderType::Compute => ShaderKind::Compute,
//            ShaderType::TessellationControl => ShaderKind::TessControl,
//            ShaderType::TessellationEvaluation => ShaderKind::TessEvaluation,
//        };
//    }
//}

impl From<ShaderType> for ShaderKind {
    fn from(value: ShaderType) -> Self {
        match value {
            ShaderType::Fragment => ShaderKind::Fragment,
            ShaderType::Vertex => ShaderKind::Vertex,
            ShaderType::Geometry => ShaderKind::Geometry,
            ShaderType::Compute => ShaderKind::Compute,
            ShaderType::TessellationControl => ShaderKind::TessControl,
            ShaderType::TessellationEvaluation => ShaderKind::TessEvaluation,
        }
    }
}

fn compile(src: &str, type_of_shader: ShaderType) -> Vec<u32> {
    let processed = preprocessor::process(src);
    let compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.add_macro_definition("EP", Some("main"));
    let binary_result = compiler
        .compile_into_spirv(
            processed.as_str(),
            type_of_shader.into(),
            "shader.glsl",
            "main",
            Some(&options),
        )
        .unwrap();
    binary_result.as_binary().to_vec()
}

mod preprocessor {
    use crate::render::consts::{MAX_LIGHTS, MAX_TEXTURES};

    const MAX_TEXTURES_IDENTIFIER: &str = "MAX_TEXTURES";
    const MAX_LIGHTS_IDENTIFIER: &str = "MAX_LIGHTS";

    pub fn process(src: &str) -> String {
        src.replace(MAX_TEXTURES_IDENTIFIER, &format!("{}", MAX_TEXTURES))
            .replace(MAX_LIGHTS_IDENTIFIER, &format!("{}", MAX_LIGHTS))
    }
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
    vert: Option<Vec<u32>>,
    frag: Option<Vec<u32>>,
    pipelines: Option<Vec<RenderPipeline>>
}

impl Clone for Shader {
    fn clone(&self) -> Self {
        if self.pipelines.is_some() {
            panic!("Cannot clone a shader that is already made!");
        }
        Shader {
            id: self.id,
            vert: self.vert.clone(),
            frag: self.frag.clone(),
            pipelines: None,
        }
    }
}

impl Shader {
    pub(crate) fn compile_glsl(code: &str, shader_type: ShaderType) -> Vec<u32> {
        compile(code, shader_type)
    }

    pub(crate) fn new(vert: Vec<u32>, frag: Vec<u32>) -> Self {
        Self {
            id: next_id("MVCore::Shader"),
            vert: Some(vert),
            frag: Some(frag),
            pipelines: None,
        }
    }

    pub(crate) fn new_glsl(vert: &str, frag: &str) -> Self {
        let v_spv = compile(vert, ShaderType::Vertex);
        let f_spv = compile(frag, ShaderType::Fragment);
        Self {
            id: next_id("MVCore::Shader"),
            vert: Some(v_spv),
            frag: Some(f_spv),
            pipelines: None,
        }
    }

    pub(crate) fn setup_pipeline(
        mut self,
        state: &State,
        layout: VertexBufferLayout<'static>,
        bind_groups: &[u8],
    ) -> Self {
        unsafe {
            if self.vert.is_none() || self.frag.is_none() {
                return self;
            }

            let vert = self.vert.take().unwrap();

            let vert = state
                .device
                .create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
                    label: Some("vert"),
                    source: Cow::from(&vert),
                });

            let frag = self.frag.take().unwrap();

            let frag = state
                .device
                .create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
                    label: Some("frag"),
                    source: Cow::from(&frag),
                });

            self.pipelines = Some(Vec::with_capacity(4));


            let nothing_pipeline = PipelineBuilder::begin(state)
                    .custom_vertex_layout(layout.clone())
                    .param(
                        PipelineBuilder::RENDER_MODE,
                        PipelineBuilder::RENDER_MODE_TRIANGLES,
                    )
                    .shader(PipelineBuilder::SHADER_VERTEX, &vert)
                    .shader(PipelineBuilder::SHADER_FRAGMENT, &frag)
                    .bind_groups(bind_groups)
                    .build();

            let stripped_pipeline = PipelineBuilder::begin(state)
                    .custom_vertex_layout(layout.clone())
                    .param(
                        PipelineBuilder::RENDER_MODE,
                        PipelineBuilder::RENDER_MODE_TRIANGLE_STRIP,
                    )
                    .shader(PipelineBuilder::SHADER_VERTEX, &vert)
                    .shader(PipelineBuilder::SHADER_FRAGMENT, &frag)
                    .bind_groups(bind_groups)
                    .build();

            let stencil_pipeline = PipelineBuilder::begin(state)
                    .custom_vertex_layout(layout.clone())
                    .param(
                        PipelineBuilder::RENDER_MODE,
                        PipelineBuilder::RENDER_MODE_TRIANGLES,
                    )
                    .param(
                        PipelineBuilder::STENCIL_MODE,
                        PipelineBuilder::WRITE_STENCIL
                    )
                    .shader(PipelineBuilder::SHADER_VERTEX, &vert)
                    .shader(PipelineBuilder::SHADER_FRAGMENT, &frag)
                    .bind_groups(bind_groups)
                    .build();

            let stencil_stripped_pipeline = PipelineBuilder::begin(state)
                    .custom_vertex_layout(layout)
                    .param(
                        PipelineBuilder::RENDER_MODE,
                        PipelineBuilder::RENDER_MODE_TRIANGLE_STRIP,
                    )
                    .param(
                        PipelineBuilder::STENCIL_MODE,
                        PipelineBuilder::WRITE_STENCIL
                    )
                    .shader(PipelineBuilder::SHADER_VERTEX, &vert)
                    .shader(PipelineBuilder::SHADER_FRAGMENT, &frag)
                    .bind_groups(bind_groups)
                    .build();

            self.pipelines.push(nothing_pipeline);
            self.pipelines.push(stripped_pipeline);
            self.pipelines.push(stencil_pipeline);
            self.pipelines.push(stencil_stripped_pipeline);
            self
        }
    }

    pub(crate) const PIPELINE_STRIPPED: u8 = 1;
    pub(crate) const PIPELINE_STENCIL: u8 = 2;

    pub(crate) fn get_pipeline(&self, flags: u8) -> Option<&RenderPipeline> {
        let is_stripped = flags & Self::PIPELINE_STRIPPED == Self::PIPELINE_STRIPPED;
        let is_stencil = flags & Self::PIPELINE_STENCIL == Self::PIPELINE_STENCIL;
        let mut i: usize = 0;
        if is_stripped { i += 1; }
        if is_stencil { i += 1; }
        if is_stripped && is_stencil { i += 1; }
        self.pipelines.expect("Shader not initialized").get(i)
    }
}

pub struct EffectShader {
    id: u64,
    shader: Option<Vec<u32>>,
    pipeline: Option<RenderPipeline>,
    uniform_size: u64,
    buffer: Option<Buffer>,
    uniform: Option<BindGroup>,
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
            uniform_size: self.uniform_size,
            buffer: None,
            uniform: None,
        }
    }
}

impl EffectShader {
    pub(crate) fn new(shader: Vec<u32>, uniform_size: u64) -> Self {
        Self {
            id: next_id("MVCore::EffectShader"),
            shader: Some(shader),
            pipeline: None,
            uniform_size,
            buffer: None,
            uniform: None,
        }
    }

    pub(crate) fn new_glsl(shader: &str, uniform_size: u64) -> Self {
        let spv = compile(shader, ShaderType::Fragment);
        Self {
            id: next_id("MVCore::EffectShader"),
            shader: Some(spv),
            pipeline: None,
            uniform_size,
            buffer: None,
            uniform: None,
        }
    }

    pub(crate) fn setup_pipeline(mut self, state: &State, bind_groups: &[u8]) -> Self {
        unsafe {
            if self.shader.is_none() {
                return self;
            }

            let vert = compile(DUMMY_VERT, ShaderType::Vertex);

            let vert = state
                .device
                .create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
                    label: Some("effect_vert"),
                    source: Cow::from(&vert),
                });

            let frag = self.shader.take().unwrap();

            let frag = state
                .device
                .create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
                    label: Some("effect_frag"),
                    source: Cow::from(&frag),
                });

            self.pipeline = Some(
                PipelineBuilder::begin(state)
                    .param(
                        PipelineBuilder::RENDER_MODE,
                        PipelineBuilder::RENDER_MODE_TRIANGLES,
                    )
                    .param(
                        PipelineBuilder::VERTEX_LAYOUT,
                        PipelineBuilder::VERTEX_LAYOUT_NONE,
                    )
                    .shader(PipelineBuilder::SHADER_VERTEX, &vert)
                    .shader(PipelineBuilder::SHADER_FRAGMENT, &frag)
                    .bind_groups(bind_groups)
                    .build(),
            );

            self.buffer = Some(state.gen_uniform_buffer_sized((self.uniform_size * 4).max(4)));

            self.uniform = Some(state.device.create_bind_group(&BindGroupDescriptor {
                label: Some("Effect custom uniforms"),
                layout: BIND_GROUPS.get(&BIND_GROUP_EFFECT_CUSTOM).unwrap(),
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: self.buffer.as_ref().unwrap().as_entire_binding(),
                }],
            }));

            self
        }
    }

    pub(crate) fn get_buffer_maker(&self) -> BufferMaker {
        BufferMaker::new(self.uniform_size as usize)
    }

    pub(crate) fn setup<F>(&self, state: &State, f: F)
    where
        F: FnOnce(&mut BufferMaker),
    {
        let mut maker = self.get_buffer_maker();
        f(&mut maker);
        maker.finish(
            state,
            self.buffer.as_ref().expect("Setting up unmade shader!"),
        );
    }

    pub(crate) fn get_uniforms(&self) -> &BindGroup {
        self.uniform.as_ref().expect("Binding unmade shader!")
    }

    pub(crate) fn get_pipeline(&self) -> &RenderPipeline {
        self.pipeline.as_ref().expect("Binding unmade shader!")
    }
}

pub struct BufferMaker {
    size: usize,
    data: Vec<f32>,
}

impl BufferMaker {
    pub(crate) fn new(size: usize) -> Self {
        BufferMaker {
            size,
            data: vec![0.0; size],
        }
    }

    pub fn set_float(&mut self, offset: usize, value: f32) {
        if offset + 1 > self.size {
            panic!("Uniform buffer index out of bounds!");
        }
        self.data[offset] = value;
    }

    pub fn set_vec2(&mut self, offset: usize, value: Vec2) {
        if offset + 2 > self.size {
            panic!("Uniform buffer index out of bounds!");
        }
        self.data[offset..offset + 2].copy_from_slice(value.to_array().as_slice());
    }

    pub fn set_vec3(&mut self, offset: usize, value: Vec3) {
        if offset + 3 > self.size {
            panic!("Uniform buffer index out of bounds!");
        }
        self.data[offset..offset + 3].copy_from_slice(value.to_array().as_slice());
    }

    pub fn set_vec4(&mut self, offset: usize, value: Vec4) {
        if offset + 4 > self.size {
            panic!("Uniform buffer index out of bounds!");
        }
        self.data[offset..offset + 4].copy_from_slice(value.to_array().as_slice());
    }

    pub fn set_mat2(&mut self, offset: usize, value: Mat2) {
        if offset + 4 > self.size {
            panic!("Uniform buffer index out of bounds!");
        }
        self.data[offset..offset + 4].copy_from_slice(value.to_cols_array().as_slice());
    }

    pub fn set_mat3(&mut self, offset: usize, value: Mat3) {
        if offset + 9 > self.size {
            panic!("Uniform buffer index out of bounds!");
        }
        self.data[offset..offset + 9].copy_from_slice(value.to_cols_array().as_slice());
    }

    pub fn set_mat4(&mut self, offset: usize, value: Mat4) {
        if offset + 16 > self.size {
            panic!("Uniform buffer index out of bounds!");
        }
        self.data[offset..offset + 16].copy_from_slice(value.to_cols_array().as_slice());
    }

    pub fn set_int(&mut self, offset: usize, value: i32) {
        if offset + 1 > self.size {
            panic!("Uniform buffer index out of bounds!");
        }
        self.data[offset] = f32::from_bits(value as u32)
    }

    pub fn set_uint(&mut self, offset: usize, value: u32) {
        if offset + 1 > self.size {
            panic!("Uniform buffer index out of bounds!");
        }
        self.data[offset] = f32::from_bits(value)
    }

    pub(crate) fn finish(self, state: &State, buffer: &Buffer) {
        state
            .queue
            .write_buffer(buffer, 0, self.data.as_slice().cast_bytes());
    }
}

pub struct Texture {
    id: u64,
    width: u32,
    height: u32,
    image: Option<Bytecode>,
    texture: Option<wgpu::Texture>,
    view: Option<TextureView>,
}

impl Texture {
    pub(crate) fn new(image: Bytecode) -> Self {
        Self {
            id: next_id("MVCore::Texture"),
            width: 0,
            height: 0,
            image: Some(image),
            texture: None,
            view: None,
        }
    }

    ///Special texture for storing runtime informations. Don't call make() on it!
    pub(crate) fn buffer(state: &State) -> Self {
        let tex = state.device.create_texture(&TextureDescriptor {
            label: Some("Buffer Texture"),
            size: Extent3d {
                width: state.config.width,
                height: state.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = tex.create_view(&TextureViewDescriptor::default());

        Self {
            id: next_id("MVCore::Texture"),
            width: state.config.width,
            height: state.config.height,
            image: None,
            texture: Some(tex),
            view: Some(view),
        }
    }

    ///Special texture for storing depth informations. Don't call make() on it!!!
    pub(crate) fn gen_depth(state: &State) -> Self {
        let tex = state.device.create_texture(&TextureDescriptor {
            label: Some("Depth Buffer"),
            size: Extent3d {
                width: state.config.width,
                height: state.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        //Change Filters here!
        let view = tex.create_view(&TextureViewDescriptor::default());

        Self {
            id: next_id("MVCore::Texture"),
            width: state.config.width,
            height: state.config.height,
            image: None,
            texture: Some(tex),
            view: Some(view),
        }
    }

    pub(crate) fn premade(texture: wgpu::Texture, view: TextureView) -> Self {
        let width = texture.width();
        let height = texture.height();
        Self {
            id: next_id("MVCore::Texture"),
            width,
            height,
            image: None,
            texture: Some(texture),
            view: Some(view),
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

        self.width = dimensions.0;
        self.height = dimensions.1;

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
            &bytes,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
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

    pub(crate) fn get_width(&self) -> u32 {
        self.width
    }

    pub(crate) fn get_height(&self) -> u32 {
        self.height
    }

    pub(crate) fn get_id(&self) -> u64 {
        self.id
    }
}

pub struct TextureRegion {
    id: u64,
    texture: Arc<Texture>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    uv: [f32; 4],
}

impl TextureRegion {
    pub fn new(texture: Arc<Texture>, x: u32, y: u32, width: u32, height: u32) -> Self {
        let w = texture.get_width() as f32;
        let h = texture.get_height() as f32;
        TextureRegion {
            id: next_id("TextureRegion"),
            texture,
            x,
            y,
            width,
            height,
            uv: [
                x as f32 / w,
                (x + width) as f32 / w,
                y as f32 / h,
                (y + height) as f32 / h,
            ],
        }
    }

    pub fn from(texture: Arc<Texture>) -> Self {
        let width = texture.get_width();
        let height = texture.get_height();
        TextureRegion {
            id: next_id("TextureRegion"),
            texture,
            x: 0,
            y: 0,
            width,
            height,
            uv: [0.0, 0.0, 1.0, 1.0],
        }
    }

    pub(crate) fn get_uv(&self) -> [f32; 4] {
        self.uv
    }

    pub(crate) fn parent(&self) -> Arc<Texture> {
        self.texture.clone()
    }
}

epecpc!(Shader, EffectShader, Texture, TextureRegion);

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

impl_bytes_glam!(
    Vec2 = 8,
    Vec3 = 12,
    Vec4 = 16,
    Mat2 = 16,
    Mat3 = 36,
    Mat4 = 64
);
