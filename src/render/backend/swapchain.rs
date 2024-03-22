use crate::render::backend::device::Device;
use crate::render::backend::vulkan::swapchain::VkSwapchain;
use crate::render::backend::Extent2D;

pub(crate) struct MVSwapchainCreateInfo {
    pub(crate) extent: Extent2D,
    pub(crate) previous: Option<Swapchain>,
    pub(crate) vsync: bool,
    pub(crate) max_frames_in_flight: u32,
}

pub(crate) enum Swapchain {
    Vulkan(VkSwapchain),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl Swapchain {
    pub(crate) fn new(device: Device, create_info: MVSwapchainCreateInfo) -> Swapchain {
        match device {
            Device::Vulkan(device) => {
                Swapchain::Vulkan(VkSwapchain::new(device, create_info.into()))
            }
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }
}
