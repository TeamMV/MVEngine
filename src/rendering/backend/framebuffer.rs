use std::sync::Arc;

use bitflags::bitflags;
use mvengine_proc_macro::graphics_item;
use crate::rendering::backend::command_buffer::CommandBuffer;
use crate::rendering::backend::device::Device;
use crate::rendering::backend::image::{AccessFlags, Image, ImageFormat, ImageLayout, ImageUsage};
use crate::rendering::backend::vulkan::framebuffer::VkFramebuffer;
use crate::rendering::backend::Extent2D;

pub enum LoadOp {
    Load,
    Clear,
    DontCare,
}

pub enum StoreOp {
    Store,
    DontCare,
}

pub struct SubpassDependency {
    pub src_subpass: u32,
    pub dst_subpass: u32,
    pub src_stage_mask: PipelineStageFlags,
    pub dst_stage_mask: PipelineStageFlags,
    pub src_access_mask: AccessFlags,
    pub dst_access_mask: AccessFlags,
    pub dependency_flags: DependencyFlags,
}

pub struct MVRenderPassCreateInfo {
    pub dependencies: Vec<SubpassDependency>,
    pub load_op: Vec<LoadOp>,
    pub store_op: Vec<StoreOp>,
    pub final_layouts: Vec<ImageLayout>,
}

pub struct MVFramebufferCreateInfo {
    pub attachment_formats: Vec<ImageFormat>,
    pub extent: Extent2D,
    pub image_usage_flags: ImageUsage,
    pub render_pass_info: Option<MVRenderPassCreateInfo>,

    pub label: Option<String>,
}

#[derive(Copy, Clone)]
pub enum ClearColor {
    Color([f32; 4]),
    Depth { depth: f32, stencil: u32 },
}

#[graphics_item(clone)]
#[derive(Clone)]
pub enum Framebuffer {
    Vulkan(Arc<VkFramebuffer>),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl Framebuffer {
    pub fn new(device: Device, create_info: MVFramebufferCreateInfo) -> Self {
        match device {
            Device::Vulkan(device) => {
                Framebuffer::Vulkan(VkFramebuffer::new(device, create_info.into()).into())
            }
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub fn begin_render_pass(
        &self,
        command_buffer: &CommandBuffer,
        clear_color: &[ClearColor],
        extent: Extent2D,
    ) {
        match self {
            Framebuffer::Vulkan(framebuffer) => framebuffer.begin_render_pass(
                command_buffer.as_vulkan(),
                clear_color,
                extent.into(),
            ),
            #[cfg(target_os = "macos")]
            Framebuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Framebuffer::DirectX => unimplemented!(),
        }
    }

    pub fn end_render_pass(&self, command_buffer: &CommandBuffer) {
        match self {
            Framebuffer::Vulkan(framebuffer) => {
                framebuffer.end_render_pass(command_buffer.as_vulkan())
            }
            #[cfg(target_os = "macos")]
            Framebuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Framebuffer::DirectX => unimplemented!(),
        }
    }

    pub fn get_image(&self, index: u32) -> Image {
        match self {
            Framebuffer::Vulkan(framebuffer) => Image::Vulkan(framebuffer.get_image(index)),
            #[cfg(target_os = "macos")]
            Framebuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Framebuffer::DirectX => unimplemented!(),
        }
    }
}

bitflags! {
    pub struct PipelineStageFlags: u32 {
        const TOP_OF_PIPE = 1 << 0;
        const DRAW_INDIRECT = 1 << 1;
        const VERTEX_INPUT = 1 << 2;
        const VERTEX_SHADER = 1 << 3;
        const TESSELLATION_CONTROL_SHADER = 1 << 4;
        const TESSELLATION_EVALUATION_SHADER = 1 << 5;
        const GEOMETRY_SHADER = 1 << 6;
        const FRAGMENT_SHADER = 1 << 7;
        const EARLY_FRAGMENT_TESTS = 1 << 8;
        const LATE_FRAGMENT_TESTS = 1 << 9;
        const COLOR_ATTACHMENT_OUTPUT = 1 << 10;
        const COMPUTE_SHADER = 1 << 11;
        const TRANSFER = 1 << 12;
        const BOTTOM_OF_PIPE = 1 << 13;
        const HOST = 1 << 14;
        const ALL_GRAPHICS = 1 << 15;
        const ALL_COMMANDS = 1 << 16;
        const TASK_SHADER_EXT = 1 << 19;
        const MESH_SHADER_EXT = 1 << 20;
        #[cfg(feature = "ray-tracing")]
        const RAY_TRACING_SHADER_KHR = 1 << 21;
        #[cfg(feature = "ray-tracing")]
        const ACCELERATION_STRUCTURE_BUILD_KHR = 1 << 25;
    }
}

bitflags! {
    pub struct DependencyFlags: u8 {
        const BY_REGION = 1 << 0;
        const VIEW_LOCAL = 1 << 1;
        const DEVICE_GROUP = 1 << 2;
    }
}
