use std::ffi::CString;
use crate::render::backend::command_buffer::CommandBuffer;
use crate::render::backend::device::Device;
use crate::render::backend::swapchain::Swapchain;
use crate::render::backend::vulkan::framebuffer::{RenderPassCreateInfo, VkFramebuffer};
use crate::render::backend::vulkan::swapchain::VkSwapchain;
use crate::render::backend::Extent2D;
use mvcore_proc_macro::graphics_item;
use std::sync::Arc;
use crate::render::backend::image::{ImageFormat, ImageLayout, ImageUsage};

pub(crate) enum LoadOp {
    Load,
    Clear,
    DontCare
}

pub(crate) enum StoreOp {
    Store,
    DontCare
}

pub(crate) struct MVRenderPassCreateInfo {
    pub(crate) dependencies: Vec<ash::vk::SubpassDependency>,
    pub(crate) load_op: Vec<LoadOp>,
    pub(crate) store_op: Vec<StoreOp>,
    pub(crate) final_layouts: Vec<ImageLayout>
}

pub(crate) struct MVFramebufferCreateInfo {
    pub(crate) attachment_formats: Vec<ImageFormat>,
    pub(crate) extent: Extent2D,
    pub(crate) image_usage_flags: ImageUsage,
    pub(crate) render_pass_info: Option<MVRenderPassCreateInfo>,

    pub(crate) label: Option<String>,
}

#[derive(Copy, Clone)]
pub(crate) enum ClearColor {
    Color([f32; 4]),
    Depth { depth: f32, stencil: u32 },
}

#[graphics_item(clone)]
#[derive(Clone)]
pub(crate) enum Framebuffer {
    Vulkan(Arc<VkFramebuffer>),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl Framebuffer {
    pub(crate) fn new(device: Device, create_info: MVFramebufferCreateInfo) -> Self {
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

    pub(crate) fn begin_render_pass(
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

    pub(crate) fn end_render_pass(&self, command_buffer: &CommandBuffer) {
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
}
