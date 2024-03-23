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
