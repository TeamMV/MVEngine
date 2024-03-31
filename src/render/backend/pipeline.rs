use crate::render::backend::command_buffer::CommandBuffer;
use crate::render::backend::descriptor_set::DescriptorSetLayout;
use crate::render::backend::device::Device;
use crate::render::backend::framebuffer::Framebuffer;
use crate::render::backend::shader::{Shader, ShaderStage};
use crate::render::backend::vulkan::pipeline::VkPipeline;
use crate::render::backend::vulkan::shader::VkShader;
use crate::render::backend::Extent2D;
use mvcore_proc_macro::graphics_item;
use mvutils::sealable;
use std::marker::PhantomData;

pub trait PipelineType {}

pub struct Graphics;
impl PipelineType for Graphics {}
pub struct Compute;
impl PipelineType for Compute {}
#[cfg(feature = "ray-tracing")]
pub struct RayTracing;
#[cfg(feature = "ray-tracing")]
impl PipelineType for RayTracing {}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum AttributeType {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Topology {
    Point,
    Line,
    LineStrip,
    Triangle,
    TriangleStrip,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CullMode {
    None,
    Front,
    Back,
    Both,
}

pub struct PushConstant {
    pub size: u32,
    pub offset: u32,
    pub shader: ShaderStage,
}

pub struct MVGraphicsPipelineCreateInfo {
    pub shaders: Vec<Shader>,
    pub attributes: Vec<AttributeType>,
    pub extent: Extent2D,
    pub topology: Topology,
    pub cull_mode: CullMode,
    pub enable_depth_test: bool,
    pub depth_clamp: bool,
    pub blending_enable: bool,
    pub descriptor_sets: Vec<DescriptorSetLayout>,
    pub push_constants: Vec<PushConstant>,
    pub framebuffer: Framebuffer,
    pub color_attachments_count: u32,

    pub label: Option<String>,
}

pub struct MVComputePipelineCreateInfo {
    pub shader: Shader,
    pub descriptor_sets: Vec<DescriptorSetLayout>,
    pub push_constants: Vec<PushConstant>,

    pub label: Option<String>,
}

#[cfg(feature = "ray-tracing")]
pub struct MVRayTracingPipelineCreateInfo {
    pub ray_gen_shaders: Vec<Shader>,
    pub closest_hit_shaders: Vec<Shader>,
    pub miss_shaders: Vec<Shader>,
    pub descriptor_sets: Vec<DescriptorSetLayout>,
    pub push_constants: Vec<PushConstant>,

    pub label: Option<String>,
}

#[graphics_item(ref)]
pub enum Pipeline<Type: PipelineType = Graphics> {
    Vulkan(VkPipeline<Type>),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl Pipeline {
    pub fn new(device: Device, create_info: MVGraphicsPipelineCreateInfo) -> Self {
        match device {
            Device::Vulkan(device) => {
                Pipeline::Vulkan(VkPipeline::<Graphics>::new(device, create_info.into()))
            }
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub fn bind(&self, command_buffer: &CommandBuffer) {
        match self {
            Pipeline::Vulkan(pipeline) => pipeline.bind(command_buffer.as_vulkan().get_handle()),
            #[cfg(target_os = "macos")]
            Pipeline::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Pipeline::DirectX => unimplemented!(),
        }
    }
}

impl Pipeline<Compute> {
    pub fn new(device: Device, create_info: MVComputePipelineCreateInfo) -> Self {
        match device {
            Device::Vulkan(device) => {
                Pipeline::Vulkan(VkPipeline::<Compute>::new(device, create_info.into()))
            }
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub fn bind(&self, command_buffer: &CommandBuffer) {
        match self {
            Pipeline::Vulkan(pipeline) => pipeline.bind(command_buffer.as_vulkan().get_handle()),
            #[cfg(target_os = "macos")]
            Pipeline::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Pipeline::DirectX => unimplemented!(),
        }
    }
}

#[cfg(feature = "ray-tracing")]
impl Pipeline<RayTracing> {
    pub fn new(device: Device, create_info: MVRayTracingPipelineCreateInfo) -> Self {
        match device {
            Device::Vulkan(device) => {
                Pipeline::Vulkan(VkPipeline::<RayTracing>::new(device, create_info.into()))
            }
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub fn bind(&self, command_buffer: &CommandBuffer) {
        match self {
            Pipeline::Vulkan(pipeline) => pipeline.bind(command_buffer.as_vulkan().get_handle()),
            #[cfg(target_os = "macos")]
            Pipeline::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Pipeline::DirectX => unimplemented!(),
        }
    }
}
