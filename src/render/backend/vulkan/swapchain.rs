use crate::render::backend::swapchain::{MVSwapchainCreateInfo, Swapchain};
use crate::render::backend::vulkan::device::VkDevice;
use mvutils::utils::TetrahedronOp;
use std::sync::Arc;

pub(crate) struct VkSwapchain {
    device: Arc<VkDevice>,

    // Swapchain Info
    color_image_format: ash::vk::Format,
    depth_image_format: ash::vk::Format,
    current_extent: ash::vk::Extent2D,
    handle: ash::vk::SwapchainKHR,
    current_frame: u32,
    in_flight_fences: Vec<ash::vk::Fence>,
    available_semaphores: Vec<ash::vk::Semaphore>,
    finished_semaphores: Vec<ash::vk::Semaphore>,
    presentable_images: Vec<ash::vk::Image>,
    presentable_image_views: Vec<ash::vk::ImageView>,
    presentable_framebuffers: Vec<ash::vk::Framebuffer>,
    presentable_render_pass: Vec<ash::vk::RenderPass>,
}

pub(crate) struct CreateInfo {
    window_extent: ash::vk::Extent2D,
    prev_swapchain: Option<VkSwapchain>,
    vsync: bool,
    max_frames_in_flight: u32,
}

impl From<MVSwapchainCreateInfo> for CreateInfo {
    fn from(value: MVSwapchainCreateInfo) -> Self {
        CreateInfo {
            window_extent: ash::vk::Extent2D {
                width: value.extent.width,
                height: value.extent.height,
            },
            prev_swapchain: value.previous.map(|s| {
                #[allow(irrefutable_let_patterns)]
                let Swapchain::Vulkan(swapchain) = s
                else {
                    unreachable!()
                };
                swapchain
            }),
            vsync: value.vsync,
            max_frames_in_flight: value.max_frames_in_flight,
        }
    }
}

struct SwapchainCapabilities {
    capabilities: ash::vk::SurfaceCapabilitiesKHR,
    formats: Vec<ash::vk::SurfaceFormatKHR>,
    present_modes: Vec<ash::vk::PresentModeKHR>,
}

impl VkSwapchain {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: CreateInfo) -> Self {
        let swapchain_capabilities = Self::get_swapchain_capabilities(device.clone());
        let present_mode = Self::choose_present_mode(
            device.clone(),
            device.get_available_present_modes(),
            create_info.vsync,
        );
        let color_format = Self::choose_swapchain_color_format(&swapchain_capabilities.formats);
        let depth_format =
            Self::choose_swapchain_depth_format(device.clone(), &swapchain_capabilities.formats);

        let mut image_count = swapchain_capabilities.capabilities.min_image_count + 1;
        if image_count < create_info.max_frames_in_flight {
            image_count += create_info.max_frames_in_flight - image_count;
        }

        let mut vk_create_info = ash::vk::SwapchainCreateInfoKHR::builder()
            .min_image_count(image_count)
            .image_format(color_format.format)
            .image_color_space(color_format.color_space)
            .image_extent(create_info.window_extent)
            .surface(device.get_surface())
            .image_array_layers(1)
            .image_usage(ash::vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(swapchain_capabilities.capabilities.current_transform)
            .composite_alpha(ash::vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true) // discards pixels that are obscured (for example behind other window)
            .build();

        if create_info.prev_swapchain.is_some() {
            vk_create_info.old_swapchain = create_info.prev_swapchain.unwrap().handle;
        }

        let indices = device.get_indices();

        let indices_vec = [
            indices.graphics_queue_index.unwrap(),
            indices.graphics_queue_index.unwrap(),
        ];

        // if graphics and present queue are the same which happens on some hardware create images in concurrent sharing mode
        if indices.present_queue_index == indices.graphics_queue_index {
            vk_create_info.image_sharing_mode = ash::vk::SharingMode::CONCURRENT;
            vk_create_info.queue_family_index_count = 2;
            vk_create_info.p_queue_family_indices = indices_vec.as_ptr();
        } else {
            vk_create_info.image_sharing_mode = ash::vk::SharingMode::EXCLUSIVE;
            vk_create_info.queue_family_index_count = 0;
            vk_create_info.p_queue_family_indices = std::ptr::null();
        }

        let swapchain = unsafe {
            device
                .get_swapchain_extension()
                .create_swapchain(&vk_create_info, None)
        }
        .unwrap_or_else(|e| {
            log::error!("Failed to create swapchain, error: {e}");
            panic!()
        });

        todo!()
    }

    fn choose_swapchain_color_format(
        available_formats: &[ash::vk::SurfaceFormatKHR],
    ) -> ash::vk::SurfaceFormatKHR {
        for format in available_formats {
            // look for UNORM in LINEAR color space
            if format.format == ash::vk::Format::R8G8B8A8_UNORM
                && format.color_space == ash::vk::ColorSpaceKHR::EXTENDED_SRGB_LINEAR_EXT
            {
                return *format;
            }
        }

        // if the desired format isn't available just return the first one
        available_formats[0]
    }

    fn choose_swapchain_depth_format(
        device: Arc<VkDevice>,
        available_formats: &[ash::vk::SurfaceFormatKHR],
    ) -> ash::vk::Format {
        device.find_supported_formats(
            &[ash::vk::Format::D16_UNORM, ash::vk::Format::D32_SFLOAT],
            ash::vk::ImageTiling::OPTIMAL,
            ash::vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }

    fn get_swapchain_capabilities(device: Arc<VkDevice>) -> SwapchainCapabilities {
        let capabilities = unsafe {
            device
                .get_surface_khr()
                .get_physical_device_surface_capabilities(
                    device.get_physical_device(),
                    device.get_surface(),
                )
        }
        .unwrap();
        let formats = unsafe {
            device
                .get_surface_khr()
                .get_physical_device_surface_formats(
                    device.get_physical_device(),
                    device.get_surface(),
                )
        }
        .unwrap();
        let present_modes = device.get_available_present_modes().clone();

        SwapchainCapabilities {
            capabilities,
            formats,
            present_modes,
        }
    }

    fn choose_present_mode(
        device: Arc<VkDevice>,
        available_present_modes: &[ash::vk::PresentModeKHR],
        vsync: bool,
    ) -> ash::vk::PresentModeKHR {
        if vsync {
            device.get_vsync_present_mode()
        } else {
            device.get_no_vsync_present_mode()
        }
    }
}
