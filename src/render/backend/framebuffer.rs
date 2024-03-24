use std::sync::Arc;
use mvcore_proc_macro::graphics_item;
use crate::render::backend::command_buffer::CommandBuffer;
use crate::render::backend::device::Device;
use crate::render::backend::Extent2D;
use crate::render::backend::swapchain::Swapchain;
use crate::render::backend::vulkan::framebuffer::VkFramebuffer;
use crate::render::backend::vulkan::swapchain::VkSwapchain;

pub(crate) struct MVFramebufferCreateInfo {

}

#[derive(Copy, Clone)]
pub(crate) enum ClearColor {
    Color([f32; 4]),
    Depth {
        depth: f32,
        stencil: u32
    }
}

#[graphics_item(clone)]
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
            Device::Vulkan(device) => Framebuffer::Vulkan(VkFramebuffer::new(device, create_info.into()).into()),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn begin_render_pass(&self, command_buffer: &CommandBuffer, clear_color: &[ClearColor], extent: Extent2D) {
        match self {
            Framebuffer::Vulkan(framebuffer) => framebuffer.begin_render_pass(command_buffer.as_vulkan(), clear_color, extent.into()),
            #[cfg(target_os = "macos")]
            Framebuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Framebuffer::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn end_render_pass(&self, command_buffer: &CommandBuffer) {
        match self {
            Framebuffer::Vulkan(framebuffer) => framebuffer.end_render_pass(command_buffer.as_vulkan()),
            #[cfg(target_os = "macos")]
            Framebuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Framebuffer::DirectX => unimplemented!(),
        }
    }
}