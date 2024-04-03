use crate::render::backend::buffer::Buffer;
use crate::render::backend::descriptor_set::DescriptorSet;
use crate::render::backend::device::{CommandPool, Device};
use crate::render::backend::image::Image;
use crate::render::backend::pipeline::{Pipeline, PipelineType};
use crate::render::backend::vulkan::buffer::VkBuffer;
use crate::render::backend::vulkan::command_buffer::VkCommandBuffer;
use crate::render::backend::Extent3D;
use mvcore_proc_macro::graphics_item;

pub enum CommandBufferLevel {
    Primary,
    Secondary,
}

pub struct MVCommandBufferCreateInfo {
    pub level: CommandBufferLevel,
    pub pool: CommandPool,

    pub label: Option<String>,
}

#[graphics_item(ref)]
pub enum CommandBuffer {
    Vulkan(VkCommandBuffer),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl CommandBuffer {
    pub fn new(device: Device, create_info: MVCommandBufferCreateInfo) -> Self {
        match device {
            Device::Vulkan(device) => {
                CommandBuffer::Vulkan(VkCommandBuffer::new(device, create_info.into()))
            }
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub fn begin(&self) {
        match self {
            CommandBuffer::Vulkan(cmd) => cmd.begin(),
            #[cfg(target_os = "macos")]
            CommandBuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            CommandBuffer::DirectX => unimplemented!(),
        }
    }

    pub fn end(&self) {
        match self {
            CommandBuffer::Vulkan(cmd) => cmd.end(),
            #[cfg(target_os = "macos")]
            CommandBuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            CommandBuffer::DirectX => unimplemented!(),
        }
    }

    pub fn write_buffer(&self, buffer: &mut Buffer, data: &[u8], offset: u64) {
        match self {
            CommandBuffer::Vulkan(cmd) => {
                buffer
                    .as_vulkan_mut()
                    .write_to_buffer(data, offset, Some(cmd.get_handle()))
            }
            #[cfg(target_os = "macos")]
            CommandBuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            CommandBuffer::DirectX => unimplemented!(),
        }
    }

    pub fn copy_buffers(
        &self,
        src: &Buffer,
        dst: &mut Buffer,
        size: u64,
        src_offset: u64,
        dst_offset: u64,
    ) {
        match self {
            CommandBuffer::Vulkan(cmd) => VkBuffer::copy_buffer(
                src.as_vulkan(),
                dst.as_vulkan_mut(),
                size,
                src_offset,
                dst_offset,
                Some(cmd.get_handle()),
            ),
            #[cfg(target_os = "macos")]
            CommandBuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            CommandBuffer::DirectX => unimplemented!(),
        }
    }

    pub fn draw(&self, vertex_count: u32, first_vertex: u32) {
        match self {
            CommandBuffer::Vulkan(cmd) => cmd.draw(vertex_count, 1, first_vertex, 0),
            #[cfg(target_os = "macos")]
            CommandBuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            CommandBuffer::DirectX => unimplemented!(),
        }
    }

    pub fn draw_instanced(
        &self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        match self {
            CommandBuffer::Vulkan(cmd) => {
                cmd.draw(vertex_count, instance_count, first_vertex, first_instance)
            }
            #[cfg(target_os = "macos")]
            CommandBuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            CommandBuffer::DirectX => unimplemented!(),
        }
    }

    pub fn draw_indexed(&self, index_count: u32, first_index: u32) {
        match self {
            CommandBuffer::Vulkan(cmd) => cmd.draw_indexed(index_count, 1, first_index, 0),
            #[cfg(target_os = "macos")]
            CommandBuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            CommandBuffer::DirectX => unimplemented!(),
        }
    }

    pub fn draw_indexed_instanced(
        &self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        first_instance: u32,
    ) {
        match self {
            CommandBuffer::Vulkan(cmd) => {
                cmd.draw_indexed(index_count, instance_count, first_index, first_instance)
            }
            #[cfg(target_os = "macos")]
            CommandBuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            CommandBuffer::DirectX => unimplemented!(),
        }
    }

    pub fn bind_vertex_buffer(&self, buffer: &Buffer) {
        match self {
            CommandBuffer::Vulkan(cmd) => cmd.bind_vertex_buffer(buffer.as_vulkan()),
            #[cfg(target_os = "macos")]
            CommandBuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            CommandBuffer::DirectX => unimplemented!(),
        }
    }

    pub fn bind_index_buffer(&self, buffer: &Buffer) {
        match self {
            CommandBuffer::Vulkan(cmd) => cmd.bind_index_buffer(buffer.as_vulkan()),
            #[cfg(target_os = "macos")]
            CommandBuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            CommandBuffer::DirectX => unimplemented!(),
        }
    }

    pub fn dispatch(&self, extent: Extent3D) {
        match self {
            CommandBuffer::Vulkan(cmd) => cmd.dispatch(extent.into()),
            #[cfg(target_os = "macos")]
            CommandBuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            CommandBuffer::DirectX => unimplemented!(),
        }
    }

    pub fn blit_image(&self, src_image: Image, dst_image: Image) {
        match self {
            CommandBuffer::Vulkan(cmd) => {
                cmd.blit_image(src_image.into_vulkan(), dst_image.into_vulkan())
            }
            #[cfg(target_os = "macos")]
            CommandBuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            CommandBuffer::DirectX => unimplemented!(),
        }
    }

    pub fn bind_descriptor_set<Type: PipelineType>(
        &self,
        pipeline: &Pipeline,
        descriptor_set: &mut DescriptorSet,
        set_index: u32,
    ) {
        descriptor_set.bind(self, pipeline, set_index);
    }
}
