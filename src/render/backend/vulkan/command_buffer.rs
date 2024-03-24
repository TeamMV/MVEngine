use std::ffi::CString;
use std::sync::Arc;
use ash::vk::Handle;
use crate::err::panic;
use crate::render::backend::command_buffer::{CommandBufferLevel, MVCommandBufferCreateInfo};
use crate::render::backend::to_ascii_cstring;
use crate::render::backend::vulkan::device::VkDevice;

pub(crate) struct CreateInfo {
    level: ash::vk::CommandBufferLevel,
    pool: ash::vk::CommandPool,

    #[cfg(debug_assertions)]
    debug_name: CString
}

impl From<MVCommandBufferCreateInfo> for CreateInfo {
    fn from(value: MVCommandBufferCreateInfo) -> Self {
        CreateInfo {
            level: match value.level {
                CommandBufferLevel::Primary => ash::vk::CommandBufferLevel::PRIMARY,
                CommandBufferLevel::Secondary => ash::vk::CommandBufferLevel::SECONDARY,
            },
            pool: value.pool.into_vulkan(),

            #[cfg(debug_assertions)]
            debug_name: to_ascii_cstring(value.label.unwrap_or("".to_string())),
        }
    }
}

pub(crate) struct VkCommandBuffer {
    device: Arc<VkDevice>,
    handle: ash::vk::CommandBuffer
}

impl VkCommandBuffer {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: CreateInfo) -> Self {
        let vk_create_info = ash::vk::CommandBufferAllocateInfo::builder()
            .level(create_info.level)
            .command_pool(create_info.pool)
            .command_buffer_count(1)
            .build();

        let buffer = unsafe { device.get_device().allocate_command_buffers(&vk_create_info) }.unwrap_or_else(|e| {
            log::error!("Failed to allocate command buffer, error: {e}");
            panic!()
        })[0];

        #[cfg(debug_assertions)]
        device.set_object_name(&ash::vk::ObjectType::BUFFER, buffer.as_raw(), create_info.debug_name.as_c_str());

        Self {
            device,
            handle: buffer
        }
    }

    pub(crate) fn from(device: Arc<VkDevice>, buffer: ash::vk::CommandBuffer) -> Self {
        Self {
            device,
            handle: buffer,
        }
    }

    pub(crate) fn begin(&self) {
        let begin_info = ash::vk::CommandBufferBeginInfo::builder()
            .build();

        unsafe { self.device.get_device().begin_command_buffer(self.handle, &begin_info)}.unwrap_or_else(|e| {
            log::error!("Failed to begin recording command buffer, error: {e}");
            panic!();
        })
    }

    pub(crate) fn end(&self) {
        unsafe { self.device.get_device().end_command_buffer(self.handle) }.unwrap_or_else(|e| {
            log::error!("Failed to end recording command buffer, error: {e}");
            panic!();
        })
    }
    
    pub(crate) fn draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        unsafe { self.device.get_device().cmd_draw(self.handle, vertex_count, instance_count, first_vertex, first_instance) };
    }

    pub(crate) fn get_handle(&self) -> ash::vk::CommandBuffer {
        self.handle
    }
}