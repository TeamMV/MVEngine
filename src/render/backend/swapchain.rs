use mvcore_proc_macro::graphics_item;
use crate::render::backend::command_buffer::CommandBuffer;
use crate::render::backend::device::Device;
use crate::render::backend::vulkan::swapchain::VkSwapchain;
use crate::render::backend::Extent2D;

pub(crate) struct MVSwapchainCreateInfo {
    pub(crate) extent: Extent2D,
    pub(crate) previous: Option<Swapchain>,
    pub(crate) vsync: bool,
    pub(crate) max_frames_in_flight: u32,
}

#[graphics_item(ref)]
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

    pub(crate) fn get_render_pass(&self) -> RenderPass {
        match self {
            Swapchain::Vulkan(swapchain) => RenderPass::Vulkan(swapchain.get_render_pass()),
            #[cfg(target_os = "macos")]
            Swapchain::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Swapchain::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn get_extent(&self) -> Extent2D {
        match self {
            Swapchain::Vulkan(swapchain) => swapchain.get_extent().into(),
            #[cfg(target_os = "macos")]
            Swapchain::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Swapchain::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn get_aspect_ratio(&self) -> f32 {
        match self {
            Swapchain::Vulkan(swapchain) => swapchain.get_aspect_ratio(),
            #[cfg(target_os = "macos")]
            Swapchain::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Swapchain::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn get_current_preset_mode(&self) -> PresentMode {
        match self {
            Swapchain::Vulkan(swapchain) => swapchain.get_current_present_mode().into(),
            #[cfg(target_os = "macos")]
            Swapchain::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Swapchain::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn submit_command_buffer(
        &mut self,
        buffer: &[CommandBuffer],
        image_index: u32,
    ) -> Result<(), SwapchainError> {
        match self {
            Swapchain::Vulkan(swapchain) => swapchain
                .submit_command_buffer(
                    &buffer
                        .iter()
                        .map(CommandBuffer::as_vulkan)
                        .collect::<Vec<_>>(),
                    image_index,
                )
                .map_err(Into::into),
            #[cfg(target_os = "macos")]
            Swapchain::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Swapchain::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn acquire_next_image(&self) -> Result<u32, SwapchainError> {
        match self {
            Swapchain::Vulkan(swapchain) => swapchain.acquire_next_image().map_err(Into::into),
            #[cfg(target_os = "macos")]
            Swapchain::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Swapchain::DirectX => unimplemented!(),
        }
    }
}

#[graphics_item(copy)]
pub(crate) enum RenderPass {
    Vulkan(ash::vk::RenderPass),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl RenderPass {}

pub(crate) enum PresentMode {
    Immediate,
    Mailbox,
    Fifo,
    FifoRelaxed,
}

pub(crate) enum SwapchainError {
    OutOfDate,
    Suboptimal,
}
