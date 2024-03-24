use std::marker::PhantomData;
use mvcore_proc_macro::graphics_item;
use mvutils::sealable;
use crate::render::backend::command_buffer::CommandBuffer;
use crate::render::backend::descriptor_set::DescriptorSetLayout;
use crate::render::backend::device::Device;
use crate::render::backend::Extent2D;
use crate::render::backend::framebuffer::Framebuffer;
use crate::render::backend::shader::{Shader, ShaderStage};
use crate::render::backend::vulkan::pipeline::VkPipeline;
use crate::render::backend::vulkan::shader::VkShader;

pub(crate) trait PipelineType {}

pub(crate) struct Graphics;
impl PipelineType for Graphics {}
pub(crate) struct Compute;
impl PipelineType for Compute {}
#[cfg(feature = "ray-tracing")]
pub(crate) struct RayTracing;
#[cfg(feature = "ray-tracing")]
impl PipelineType for RayTracing {}

pub(crate) enum VertexInputRate {
    Vertex,
    Instance,
}

pub(crate) struct Binding {
    pub(crate) binding: u32,
    pub(crate) stride: u32,
    pub(crate) input_rate: VertexInputRate,
}

pub(crate) enum AttributeType {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
}

pub(crate) struct Attribute {
    pub(crate) location: u32,
    pub(crate) binding: u32,
    pub(crate) offset: u32,
    pub(crate) ty: AttributeType
}

pub(crate) enum Topology {
    Line,
    LineStrip,
    Triangle,
    TriangleStrip,
}

pub(crate) enum CullMode {
    None,
    Front,
    Back,
    Both,
}

pub(crate) struct PushConstant {
    pub(crate) size: u32,
    pub(crate) offset: u32,
    pub(crate) shader: ShaderStage,
}

pub(crate) struct MVGraphicsPipelineCreateInfo {
    pub(crate) shaders: Vec<Shader>,
    pub(crate) bindings: Vec<Binding>,
    pub(crate) attributes: Vec<Attribute>,
    pub(crate) extent: Extent2D,
    pub(crate) topology: Topology,
    pub(crate) cull_mode: CullMode,
    pub(crate) enable_depth_test: bool,
    pub(crate) depth_clamp: bool,
    pub(crate) blending_enable: bool,
    pub(crate) descriptor_sets: Vec<DescriptorSetLayout>,
    pub(crate) push_constants: Vec<PushConstant>,
    pub(crate) framebuffer: Framebuffer,
    pub(crate) color_attachments_count: u32,

    pub(crate) label: Option<String>,
}

pub(crate) struct MVComputePipelineCreateInfo {
    pub(crate) shader: Shader,
    pub(crate) descriptor_sets: Vec<DescriptorSetLayout>,
    pub(crate) push_constants: Vec<PushConstant>,

    pub(crate) label: Option<String>,
}

#[cfg(feature = "ray-tracing")]
pub(crate) struct MVRayTracingPipelineCreateInfo {
    pub(crate) ray_gen_shaders: Vec<Shader>,
    pub(crate) closest_hit_shaders: Vec<Shader>,
    pub(crate) miss_shaders: Vec<Shader>,
    pub(crate) descriptor_sets: Vec<DescriptorSetLayout>,
    pub(crate) push_constants: Vec<PushConstant>,

    pub(crate) label: Option<String>,
}

#[graphics_item(ref)]
pub(crate) enum Pipeline<Type: PipelineType = Graphics> {
    Vulkan(VkPipeline<Type>),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl Pipeline {
    pub(crate) fn new(device: Device, create_info: MVGraphicsPipelineCreateInfo) -> Self {
        match device {
            Device::Vulkan(device) => Pipeline::Vulkan(VkPipeline::<Graphics>::new(device, create_info.into())),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn bind(&self, command_buffer: &CommandBuffer) {
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
    pub(crate) fn new(device: Device, create_info: MVComputePipelineCreateInfo) -> Self {
        match device {
            Device::Vulkan(device) => Pipeline::Vulkan(VkPipeline::<Compute>::new(device, create_info.into())),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn bind(&self, command_buffer: &CommandBuffer) {
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
    pub(crate) fn new(device: Device, create_info: MVRayTracingPipelineCreateInfo) -> Self {
        match device {
            Device::Vulkan(device) => Pipeline::Vulkan(VkPipeline::<RayTracing>::new(device, create_info.into())),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn bind(&self, command_buffer: &CommandBuffer) {
        match self {
            Pipeline::Vulkan(pipeline) => pipeline.bind(command_buffer.as_vulkan().get_handle()),
            #[cfg(target_os = "macos")]
            Pipeline::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Pipeline::DirectX => unimplemented!(),
        }
    }
}