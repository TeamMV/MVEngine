use mvcore_proc_macro::graphics_item;
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
    fn some_command(&self) {
        match self {
            CommandBuffer::Vulkan(buffer) => vk::some_command(*buffer),
            #[cfg(target_os = "macos")]
            CommandBuffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            CommandBuffer::DirectX => unimplemented!(),
        }
    }
}
