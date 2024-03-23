use mvcore_proc_macro::graphics_item;
use crate::render::backend::buffer::Buffer;
use crate::render::backend::vulkan::buffer::VkBuffer;
use crate::render::backend::vulkan::command_buffer as vk;

#[graphics_item(copy)]
pub(crate) enum CommandBuffer {
    Vulkan(ash::vk::CommandBuffer),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl CommandBuffer {
    pub(crate) fn write_buffer(&self, buffer: &Buffer, data: &[u8], offset: u64) {
        match self {
            CommandBuffer::Vulkan(cmd) => buffer.as_vulkan().write_to_buffer(data, offset, Some(*cmd)),
            #[cfg(target_os = "macos")]
            CommandBuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            CommandBuffer::DirectX => unimplemented!(),
        }
    }

     pub(crate) fn copy_buffers(&self, src: &Buffer, dst: &Buffer, size: u64) {
         match self {
             CommandBuffer::Vulkan(cmd) => VkBuffer::copy_buffer(src.as_vulkan(), dst.as_vulkan(), size, Some(*cmd)),
             #[cfg(target_os = "macos")]
             CommandBuffer::Metal => unimplemented!(),
             #[cfg(target_os = "windows")]
             CommandBuffer::DirectX => unimplemented!(),
         }
     }
}
