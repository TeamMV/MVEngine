use crate::render::backend::command_buffer::CommandBuffer;
use crate::render::backend::vulkan::command_buffer::VkCommandBuffer;
use crate::render::backend::Backend;
use bitflags::bitflags;
use mvcore_proc_macro::graphics_item;
use mvutils::version::Version;
use std::sync::Arc;

use crate::render::backend::vulkan::device::VkDevice;

pub(crate) struct MVDeviceCreateInfo {
    pub(crate) app_name: String,
    pub(crate) app_version: Version,
    pub(crate) engine_name: String,
    pub(crate) engine_version: Version,

    pub(crate) device_extensions: Extensions,
}

#[graphics_item(clone)]
#[derive(Clone)]
pub(crate) enum Device {
    Vulkan(Arc<VkDevice>),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl Device {
    pub(crate) fn new(
        backend: Backend,
        create_info: MVDeviceCreateInfo,
        window: &winit::window::Window,
    ) -> Self {
        match backend {
            Backend::Vulkan => Device::Vulkan(VkDevice::new(create_info.into(), window).into()),
            #[cfg(target_os = "macos")]
            Backend::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Backend::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn begin_single_time_command(&self, pool: CommandPool) -> CommandBuffer {
        match self {
            Device::Vulkan(device) => CommandBuffer::Vulkan(VkCommandBuffer::from(
                device.clone(),
                device.begin_single_time_command(pool.as_vulkan()),
            )),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn end_single_time_command(
        &self,
        buffer: CommandBuffer,
        pool: CommandPool,
        queue: Queue,
    ) {
        match self {
            Device::Vulkan(device) => device.end_single_time_command(
                buffer.as_vulkan().get_handle(),
                pool.as_vulkan(),
                queue.as_vulkan(),
            ),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn get_graphics_command_pool(&self) -> CommandPool {
        match self {
            Device::Vulkan(device) => CommandPool::Vulkan(device.get_graphics_command_pool()),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn get_compute_command_pool(&self) -> CommandPool {
        match self {
            Device::Vulkan(device) => CommandPool::Vulkan(device.get_compute_command_pool()),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn get_graphics_queue(&self) -> Queue {
        match self {
            Device::Vulkan(device) => Queue::Vulkan(device.get_graphics_queue()),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn get_compute_queue(&self) -> Queue {
        match self {
            Device::Vulkan(device) => Queue::Vulkan(device.get_compute_queue()),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn get_present_queue(&self) -> Queue {
        match self {
            Device::Vulkan(device) => Queue::Vulkan(device.get_present_queue()),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn wait_idle(&self) {
        match self {
            Device::Vulkan(device) => device.wait_idle(),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }
}

#[graphics_item(copy)]
pub(crate) enum CommandPool {
    Vulkan(ash::vk::CommandPool),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

#[graphics_item(copy)]
pub(crate) enum Queue {
    Vulkan(ash::vk::Queue),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

bitflags! {
    pub struct Extensions: u64 {
        const MULTIVIEW = 1;
        const DESCRIPTOR_INDEXING = 1 << 1;
        const SHADER_F16 = 1 << 2;
        const DRAW_INDIRECT_COUNT = 1 << 3;
        const RAY_TRACING = 1 << 4;
        const TEXTURE_COMPRESSION_ASTC_HDR = 1 << 5;
    }
}
