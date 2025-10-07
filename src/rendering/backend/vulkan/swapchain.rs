use crate::rendering::backend::swapchain::{
    MVSwapchainCreateInfo, PresentMode, Swapchain, SwapchainError,
};
use crate::rendering::backend::vulkan::device::VkDevice;
use crate::rendering::backend::vulkan::framebuffer::VkFramebuffer;
use crate::rendering::backend::vulkan::image::VkImage;
use crate::rendering::backend::{Extent2D, Extent3D};
use ash::vk::SwapchainPresentScalingCreateInfoEXT;
use std::ops::Not;
use std::sync::Arc;

pub struct VkSwapchain {
    device: Arc<VkDevice>,

    color_image_format: ash::vk::Format,
    depth_image_format: ash::vk::Format,
    current_extent: ash::vk::Extent2D,
    handle: ash::vk::SwapchainKHR,
    current_frame: u32,
    current_image_index: u32,
    image_count: u32,
    in_flight_fences: Vec<ash::vk::Fence>,
    wait_semaphores: Vec<ash::vk::Semaphore>,
    signal_semaphores: Vec<ash::vk::Semaphore>,
    presentable_framebuffers: Vec<Arc<VkFramebuffer>>,
    present_mode: ash::vk::PresentModeKHR,
    max_frames_in_flight: u32,
    extent: ash::vk::Extent2D,
}

pub(crate) struct CreateInfo {
    window_extent: ash::vk::Extent2D,
    prev_swapchain: Option<VkSwapchain>,
    vsync: bool,
    max_frames_in_flight: u32,
}

impl From<Extent2D> for ash::vk::Extent2D {
    fn from(value: Extent2D) -> Self {
        ash::vk::Extent2D {
            width: value.width,
            height: value.height,
        }
    }
}

impl From<ash::vk::Extent2D> for Extent2D {
    fn from(value: ash::vk::Extent2D) -> Self {
        Extent2D {
            width: value.width,
            height: value.height,
        }
    }
}

impl From<Extent3D> for ash::vk::Extent3D {
    fn from(value: Extent3D) -> Self {
        ash::vk::Extent3D {
            width: value.width,
            height: value.height,
            depth: value.depth,
        }
    }
}

impl From<ash::vk::Extent3D> for Extent3D {
    fn from(value: ash::vk::Extent3D) -> Self {
        Extent3D {
            width: value.width,
            height: value.height,
            depth: value.depth,
        }
    }
}

impl From<MVSwapchainCreateInfo> for CreateInfo {
    fn from(value: MVSwapchainCreateInfo) -> Self {
        CreateInfo {
            window_extent: value.extent.into(),
            prev_swapchain: value.previous.map(Swapchain::into_vulkan),
            vsync: value.vsync,
            max_frames_in_flight: value.max_frames_in_flight,
        }
    }
}
impl From<ash::vk::Result> for SwapchainError {
    fn from(value: ash::vk::Result) -> Self {
        match value {
            ash::vk::Result::ERROR_OUT_OF_DATE_KHR => SwapchainError::OutOfDate,
            ash::vk::Result::SUBOPTIMAL_KHR => SwapchainError::Suboptimal,
            _ => {
                log::error!("vkAcquireNextImageKHR failed, error: {value}");
                panic!()
            }
        }
    }
}

impl From<ash::vk::PresentModeKHR> for PresentMode {
    fn from(value: ash::vk::PresentModeKHR) -> Self {
        match value {
            ash::vk::PresentModeKHR::IMMEDIATE => PresentMode::Immediate,
            ash::vk::PresentModeKHR::MAILBOX => PresentMode::Mailbox,
            ash::vk::PresentModeKHR::FIFO => PresentMode::Fifo,
            ash::vk::PresentModeKHR::FIFO_RELAXED => PresentMode::FifoRelaxed,
            _ => PresentMode::Fifo,
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
        if image_count > create_info.max_frames_in_flight {
            image_count = create_info.max_frames_in_flight;
        }
        if image_count < swapchain_capabilities.capabilities.min_image_count {
            image_count = swapchain_capabilities.capabilities.min_image_count;
        }

        let mut vk_create_info = ash::vk::SwapchainCreateInfoKHR::builder()
            .min_image_count(image_count)
            .image_format(color_format.format)
            .image_color_space(color_format.color_space)
            .image_extent(create_info.window_extent)
            .surface(device.get_surface())
            .image_array_layers(1)
            .image_usage(
                ash::vk::ImageUsageFlags::COLOR_ATTACHMENT | ash::vk::ImageUsageFlags::TRANSFER_DST,
            )
            .pre_transform(swapchain_capabilities.capabilities.current_transform)
            .composite_alpha(ash::vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true); // discards pixels that are obscured (for example behind other window);

        let scaling = SwapchainPresentScalingCreateInfoEXT::builder();

        if let Some(old) = create_info.prev_swapchain.as_ref() {
            vk_create_info.old_swapchain = old.handle;
        }

        let indices = device.get_indices();

        let indices_vec = [
            indices.graphics_queue_index.unwrap(),
            indices.compute_queue_index.unwrap(),
        ];

        // if graphics and present queue are the same which happens on some hardware create images in exclusive sharing mode
        if indices.present_queue_index != indices.graphics_queue_index {
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

        let images = unsafe {
            device
                .get_swapchain_extension()
                .get_swapchain_images(swapchain)
        }
        .unwrap_or_else(|e| {
            log::error!("Couldn't get swapchain images. error: {e}");
            panic!();
        });

        let render_pass = Self::create_render_pass(&device, color_format.format);

        let framebuffers = images
            .into_iter()
            .map(|image| {
                let view_create_info = ash::vk::ImageViewCreateInfo::builder()
                    .image(image)
                    .view_type(ash::vk::ImageViewType::TYPE_2D)
                    .format(color_format.format)
                    .subresource_range(ash::vk::ImageSubresourceRange {
                        aspect_mask: ash::vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });

                let view = unsafe {
                    device
                        .get_device()
                        .create_image_view(&view_create_info, None)
                }
                .unwrap_or_else(|e| {
                    log::error!("Create image view failed, error: {e}");
                    panic!()
                });

                let vk_image = VkImage {
                    device: device.clone(),
                    handle: image,
                    image_views: vec![view],
                    memory: None,
                    format: color_format.format,
                    aspect: ash::vk::ImageAspectFlags::COLOR,
                    tiling: ash::vk::ImageTiling::OPTIMAL,
                    layer_count: 1,
                    image_type: ash::vk::ImageType::TYPE_2D,
                    size: create_info.window_extent,
                    mip_level_count: 0,
                    usage: ash::vk::ImageUsageFlags::SAMPLED,
                    memory_properties: ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
                    layout: ash::vk::ImageLayout::UNDEFINED.into(),
                    memory_usage_flags: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
                    drop: false,
                }
                .into();

                Arc::new(VkFramebuffer::from(
                    device.clone(),
                    vk_image,
                    render_pass,
                    create_info.window_extent,
                ))
            })
            .collect();

        let (wait_semaphores, signal_semaphores, in_flight_fences) =
            Self::create_sync_objects(&device, create_info.max_frames_in_flight);

        Self {
            device,
            color_image_format: color_format.format,
            depth_image_format: depth_format,
            current_extent: create_info.window_extent,
            handle: swapchain,
            current_frame: 0,
            current_image_index: 0,
            in_flight_fences,
            wait_semaphores,
            signal_semaphores,
            presentable_framebuffers: framebuffers,
            present_mode,
            max_frames_in_flight: create_info.max_frames_in_flight,
            extent: create_info.window_extent,
            image_count,
        }
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

    fn create_render_pass(
        device: &Arc<VkDevice>,
        color_format: ash::vk::Format,
    ) -> ash::vk::RenderPass {
        let color_attachment = ash::vk::AttachmentDescription::builder()
            .format(color_format)
            .samples(ash::vk::SampleCountFlags::TYPE_1) // for now we'll use only 1 sample, not sure if we want to change that in future
            .load_op(ash::vk::AttachmentLoadOp::CLEAR)
            .store_op(ash::vk::AttachmentStoreOp::STORE)
            .stencil_load_op(ash::vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(ash::vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(ash::vk::ImageLayout::UNDEFINED)
            .final_layout(ash::vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = [ash::vk::AttachmentReference {
            attachment: 0,
            layout: ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];

        let subpass = ash::vk::SubpassDescription::builder()
            .pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_ref);

        let dependency = [ash::vk::SubpassDependency {
            src_subpass: ash::vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_stage_mask: ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | ash::vk::PipelineStageFlags::FRAGMENT_SHADER,
            src_access_mask: ash::vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_access_mask: ash::vk::AccessFlags::COLOR_ATTACHMENT_READ,
            dependency_flags: ash::vk::DependencyFlags::empty(),
        }];

        let attachments = [*color_attachment];
        let subpasses = [*subpass];
        let render_pass_create_info = ash::vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependency);

        unsafe {
            device
                .get_device()
                .create_render_pass(&render_pass_create_info, None)
        }
        .unwrap_or_else(|e| {
            log::error!("Failed to create swapchain rendering pass! error: {e}");
            panic!();
        })
    }

    fn create_sync_objects(
        device: &Arc<VkDevice>,
        max_frames_in_flight: u32,
    ) -> (
        Vec<ash::vk::Semaphore>,
        Vec<ash::vk::Semaphore>,
        Vec<ash::vk::Fence>,
    ) {
        let semaphore_create_info = ash::vk::SemaphoreCreateInfo::builder();

        let fence_create_info =
            ash::vk::FenceCreateInfo::builder().flags(ash::vk::FenceCreateFlags::SIGNALED);

        let mut wait_semaphores = Vec::new();
        let mut signal_semaphores = Vec::new();
        let mut in_flight_fences = Vec::new();
        for i in 0..max_frames_in_flight {
            wait_semaphores.push(
                unsafe {
                    device
                        .get_device()
                        .create_semaphore(&semaphore_create_info, None)
                }
                .unwrap_or_else(|e| {
                    log::error!("Failed to create wait semaphore, error {e}");
                    panic!()
                }),
            );

            signal_semaphores.push(
                unsafe {
                    device
                        .get_device()
                        .create_semaphore(&semaphore_create_info, None)
                }
                .unwrap_or_else(|e| {
                    log::error!("Failed to create signal semaphore, error {e}");
                    panic!()
                }),
            );

            in_flight_fences.push(
                unsafe { device.get_device().create_fence(&fence_create_info, None) }
                    .unwrap_or_else(|e| {
                        log::error!("Failed to create fence, error {e}");
                        panic!()
                    }),
            );
        }

        (wait_semaphores, signal_semaphores, in_flight_fences)
    }

    pub(crate) fn get_framebuffer(&self, index: usize) -> Arc<VkFramebuffer> {
        self.presentable_framebuffers[index].clone()
    }

    pub(crate) fn get_framebuffers(&self) -> Vec<Arc<VkFramebuffer>> {
        self.presentable_framebuffers.clone()
    }

    pub(crate) fn get_current_framebuffer(&self) -> Arc<VkFramebuffer> {
        self.presentable_framebuffers[self.current_image_index as usize].clone()
    }

    pub(crate) fn get_extent(&self) -> ash::vk::Extent2D {
        self.current_extent
    }

    pub(crate) fn get_aspect_ratio(&self) -> f32 {
        self.current_extent.width as f32 / self.current_extent.height as f32
    }

    pub(crate) fn get_current_present_mode(&self) -> ash::vk::PresentModeKHR {
        self.present_mode
    }

    pub(crate) fn submit_command_buffer(
        &mut self,
        buffer: &[ash::vk::CommandBuffer],
        image_index: u32,
    ) -> Result<(), ash::vk::Result> {
        let wait_semaphores = [self.wait_semaphores[self.current_frame as usize]];
        let signal_semaphores = [self.signal_semaphores[self.current_frame as usize]];
        let wait_stages = [ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let swapchain = [self.handle];
        let image_indices = [image_index];
        let submit_info = ash::vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .signal_semaphores(&signal_semaphores)
            .command_buffers(buffer)
            .wait_dst_stage_mask(&wait_stages);

        let vk_info = [*submit_info];
        unsafe {
            self.device.get_device().queue_submit(
                self.device.get_graphics_queue(),
                &vk_info,
                self.in_flight_fences[self.current_frame as usize],
            )
        }?;

        let present_info = ash::vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchain)
            .image_indices(&image_indices);

        self.current_frame = (self.current_frame + 1) % self.max_frames_in_flight;

        let suboptimal = unsafe {
            self.device
                .get_swapchain_extension()
                .queue_present(self.device.get_present_queue(), &present_info)
        }?;

        suboptimal
            .not()
            .then_some(())
            .ok_or(ash::vk::Result::SUBOPTIMAL_KHR)
    }

    pub(crate) fn acquire_next_image(&mut self) -> Result<u32, ash::vk::Result> {
        let fences = [self.in_flight_fences[self.current_frame as usize]];
        unsafe {
            self.device
                .get_device()
                .wait_for_fences(&fences, true, u64::MAX)
        }
        .unwrap();
        unsafe { self.device.get_device().reset_fences(&fences) }.unwrap();

        let (image, suboptimal) = unsafe {
            self.device.get_swapchain_extension().acquire_next_image(
                self.handle,
                u64::MAX,
                self.wait_semaphores[self.current_frame as usize],
                ash::vk::Fence::null(),
            )
        }?;

        self.current_image_index = image;

        suboptimal
            .not()
            .then_some(image)
            .ok_or(ash::vk::Result::SUBOPTIMAL_KHR)
    }

    pub(crate) fn get_current_frame(&self) -> u32 {
        self.current_frame
    }

    pub(crate) fn get_image_count(&self) -> u32 {
        self.image_count
    }

    pub(crate) fn get_current_image_index(&self) -> u32 {
        self.current_image_index
    }

    pub(crate) fn get_max_frames_in_flight(&self) -> u32 {
        self.max_frames_in_flight
    }
}

impl Drop for VkSwapchain {
    fn drop(&mut self) {
        unsafe {
            self.device
                .get_swapchain_extension()
                .destroy_swapchain(self.handle, None);
            self.device
                .get_device()
                .destroy_render_pass(self.presentable_framebuffers[0].get_render_pass(), None);
        };

        for i in 0..self.max_frames_in_flight {
            unsafe {
                self.device
                    .get_device()
                    .destroy_fence(self.in_flight_fences[i as usize], None);
                self.device
                    .get_device()
                    .destroy_semaphore(self.signal_semaphores[i as usize], None);
                self.device
                    .get_device()
                    .destroy_semaphore(self.wait_semaphores[i as usize], None);
            }
        }
    }
}
