use std::ffi::CString;
use crate::render::backend::framebuffer::{ClearColor, LoadOp, MVFramebufferCreateInfo, MVRenderPassCreateInfo, StoreOp};
use crate::render::backend::vulkan::command_buffer::VkCommandBuffer;
use crate::render::backend::vulkan::device::VkDevice;
use std::sync::Arc;
use ash::vk::Handle;
use crate::render::backend::image::ImageType;
use crate::render::backend::to_ascii_cstring;
use crate::render::backend::vulkan::image::VkImage;

impl From<ClearColor> for ash::vk::ClearValue {
    fn from(value: ClearColor) -> Self {
        match value {
            ClearColor::Color(c) => ash::vk::ClearValue {
                color: ash::vk::ClearColorValue { float32: c },
            },
            ClearColor::Depth { depth, stencil } => ash::vk::ClearValue {
                depth_stencil: ash::vk::ClearDepthStencilValue { depth, stencil },
            },
        }
    }
}

pub(crate) struct VkFramebuffer {
    device: Arc<VkDevice>,
    images: Vec<Arc<VkImage>>,
    handle: ash::vk::Framebuffer,
    render_pass: ash::vk::RenderPass,

    extent: ash::vk::Extent2D,
    attachment_formats: Vec<ash::vk::Format>,
    drop_render_pass: bool,
    final_layouts: Vec<ash::vk::ImageLayout>
}

pub(crate) struct RenderPassCreateInfo {
    dependencies: Vec<ash::vk::SubpassDependency>,
    load_op: Vec<ash::vk::AttachmentLoadOp>,
    store_op: Vec<ash::vk::AttachmentStoreOp>,
    final_layouts: Vec<ash::vk::ImageLayout>
}

impl RenderPassCreateInfo {
    pub(crate) fn default() -> Self {
        Self {
            dependencies: vec![],
            load_op: vec![],
            store_op: vec![],
            final_layouts: vec![],
        }
    }
}

pub(crate) struct CreateInfo {
    attachment_formats: Vec<ash::vk::Format>,
    extent: ash::vk::Extent2D,
    image_usage_flags: ash::vk::ImageUsageFlags,
    render_pass_info: Option<RenderPassCreateInfo>,

    #[cfg(debug_assertions)]
    debug_name: CString
}

impl From<LoadOp> for ash::vk::AttachmentLoadOp {
    fn from(value: LoadOp) -> Self {
        match value {
            LoadOp::Load => ash::vk::AttachmentLoadOp::LOAD,
            LoadOp::Clear => ash::vk::AttachmentLoadOp::CLEAR,
            LoadOp::DontCare => ash::vk::AttachmentLoadOp::DONT_CARE,
        }
    }
}

impl From<StoreOp> for ash::vk::AttachmentStoreOp {
    fn from(value: StoreOp) -> Self {
        match value {
            StoreOp::Store => ash::vk::AttachmentStoreOp::STORE,
            StoreOp::DontCare => ash::vk::AttachmentStoreOp::DONT_CARE,
        }
    }
}

impl From<MVRenderPassCreateInfo> for RenderPassCreateInfo {
    fn from(value: MVRenderPassCreateInfo) -> Self {
        RenderPassCreateInfo {
            dependencies: value.dependencies,
            load_op: value.load_op.into_iter().map(Into::into).collect(),
            store_op: value.store_op.into_iter().map(Into::into).collect(),
            final_layouts: value.final_layouts.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<MVFramebufferCreateInfo> for CreateInfo {
    fn from(value: MVFramebufferCreateInfo) -> Self {
        CreateInfo {
            attachment_formats: value.attachment_formats.into_iter().map(Into::into).collect(),
            extent: value.extent.into(),
            image_usage_flags: ash::vk::ImageUsageFlags::from_raw(value.image_usage_flags.bits() as u32),
            render_pass_info: value.render_pass_info.map(Into::into),

            #[cfg(debug_assertions)]
            debug_name: to_ascii_cstring(value.label.unwrap_or_default()),
        }
    }
}

impl VkFramebuffer {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: CreateInfo) -> Self {
        let mut images = Vec::new();
        let mut image_views = Vec::new();

        for image_format in &create_info.attachment_formats {
            match *image_format {
                //
                // These are the formats that we'll ever gonna use btw
                //
                ash::vk::Format::R32G32B32A32_SFLOAT |
                ash::vk::Format::R16G16B16A16_SFLOAT |
                ash::vk::Format::R8G8B8A8_UNORM |

                ash::vk::Format::R16G16B16_SFLOAT |
                ash::vk::Format::R16G16B16_UNORM |
                ash::vk::Format::R8G8B8_UNORM |

                ash::vk::Format::R32G32_SFLOAT |
                ash::vk::Format::R16G16_UNORM |
                ash::vk::Format::R8G8_UNORM |

                ash::vk::Format::R8_UNORM => {
                    let image = Self::create_color_attachment(device.clone(), create_info.extent, *image_format, create_info.image_usage_flags);
                    image_views.push(image.get_view(0));
                    images.push(image);
                }
                ash::vk::Format::D32_SFLOAT |
                ash::vk::Format::D16_UNORM |
                ash::vk::Format::D16_UNORM_S8_UINT |
                ash::vk::Format::D24_UNORM_S8_UINT => {
                    let image = Self::create_depth_attachment(device.clone(), create_info.extent, *image_format, create_info.image_usage_flags);
                    image_views.push(image.get_view(0));
                    images.push(image);
                }
                _ => {
                    log::error!("Trying to use unsupported format!");
                    panic!();
                }
            }
        }

        let render_pass = if let Some(render_pass_info) = &create_info.render_pass_info {
             Self::create_render_pass(device.clone(), &create_info.attachment_formats, render_pass_info, images.len() as u32)
        } else {
            Self::create_render_pass(device.clone(), &create_info.attachment_formats, &RenderPassCreateInfo::default(), images.len() as u32)
        };

        let framebuffer_create_info = ash::vk::FramebufferCreateInfo::builder()
            .attachment_count(images.len() as u32)
            .render_pass(render_pass)
            .width(create_info.extent.width)
            .height(create_info.extent.height)
            .layers(1)
            .attachments(&image_views);

        let handle = unsafe { device.get_device().create_framebuffer(&framebuffer_create_info, None)}.unwrap_or_else(|e| {
            log::error!("Failed to create framebuffer, error: {e}");
            panic!();
        });

        #[cfg(debug_assertions)]
        device.set_object_name(&ash::vk::ObjectType::FRAMEBUFFER, handle.as_raw(), create_info.debug_name.as_c_str());

        let final_layouts = if let Some(render_pass_info) = create_info.render_pass_info {
            render_pass_info.final_layouts
        } else {
            images.iter().enumerate().map(|(index, image)| {
                let depth = Self::is_depth_format(image.get_format(index as u32));
                if depth {
                    ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
                } else {
                    ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
                }
            }).collect()
        };

        let images = images.into_iter().map(|vk_image| {
            vk_image.into()
        }).collect();

        Self {
            device,
            images,
            handle,
            render_pass,
            extent: create_info.extent,
            attachment_formats: create_info.attachment_formats,
            drop_render_pass: false,
            final_layouts: final_layouts,
        }
    }

    fn is_depth_format(format: ash::vk::Format) -> bool {
        match format {
            ash::vk::Format::D32_SFLOAT => true,
            ash::vk::Format::D16_UNORM => true,
            ash::vk::Format::D16_UNORM_S8_UINT => true,
            ash::vk::Format::D24_UNORM_S8_UINT => true,
            _ => false
        }
    }

    fn create_color_attachment(device: Arc<VkDevice>, extent: ash::vk::Extent2D, format: ash::vk::Format, image_usage_flag: ash::vk::ImageUsageFlags) -> VkImage {
        let image_create_info = crate::render::backend::vulkan::image::CreateInfo {
            size: extent,
            format,
            usage: ash::vk::ImageUsageFlags::COLOR_ATTACHMENT | ash::vk::ImageUsageFlags::SAMPLED | image_usage_flag,
            memory_properties: ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
            aspect: ash::vk::ImageAspectFlags::COLOR,
            tiling: ash::vk::ImageTiling::OPTIMAL,
            layer_count: 1,
            image_type: ImageType::Image2D,
            cubemap: false,
            memory_usage_flags: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
            data: None,
            debug_name: CString::new("FRAMEBUFFER COLOR IMAGE").unwrap(), // TODO
        };

        VkImage::new(device.clone(), image_create_info)
    }

    fn create_depth_attachment(device: Arc<VkDevice>, extent: ash::vk::Extent2D, format: ash::vk::Format, image_usage_flag: ash::vk::ImageUsageFlags) -> VkImage {
        let image_create_info = crate::render::backend::vulkan::image::CreateInfo {
            size: extent,
            format,
            usage: ash::vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | ash::vk::ImageUsageFlags::SAMPLED | image_usage_flag,
            memory_properties: ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
            aspect: ash::vk::ImageAspectFlags::DEPTH,
            tiling: ash::vk::ImageTiling::OPTIMAL,
            layer_count: 1,
            image_type: ImageType::Image2D,
            cubemap: false,
            memory_usage_flags: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
            data: None,
            debug_name: CString::new("FRAMEBUFFER DEPTH IMAGE").unwrap(), // TODO
        };

        VkImage::new(device.clone(), image_create_info)
    }

    fn create_render_pass(device: Arc<VkDevice>, attachment_formats: &[ash::vk::Format], render_pass_create_info: &RenderPassCreateInfo, attachment_count: u32) -> ash::vk::RenderPass {
        let use_final_layouts = if !render_pass_create_info.final_layouts.is_empty() {
            #[cfg(debug_assertions)]
            if (render_pass_create_info.final_layouts.len() as u32) < attachment_count {
                log::error!("You have to specify final layout for all attachments!");
                panic!();
            };

            true
        } else {
            false
        };

        let use_loads_op = if !render_pass_create_info.load_op.is_empty() {
            #[cfg(debug_assertions)]
            if (render_pass_create_info.load_op.len() as u32) < attachment_count {
                log::error!("You have to specify load op for all attachments!");
                panic!();
            };

            true
        } else {
            false
        };

        let use_store_op = if !render_pass_create_info.store_op.is_empty() {
            #[cfg(debug_assertions)]
            if (render_pass_create_info.store_op.len() as u32) < attachment_count {
                log::error!("You have to specify final layout for all attachments!");
                panic!();
            };

            true
        } else {
            false
        };

        let mut descriptions = Vec::new();
        let mut references = Vec::new();
        let mut depth_reference = ash::vk::AttachmentReference::default();

        let mut depth_attachment_count = 0;
        for (index, format) in attachment_formats.iter().enumerate() {
            let depth = Self::is_depth_format(*format);
            if depth_attachment_count > 1 {
                log::error!("Can't have more than one depth attachment in framebuffer!");
                panic!();
            }

            let load_op = if use_loads_op {
                render_pass_create_info.load_op[index]
            } else {
                ash::vk::AttachmentLoadOp::CLEAR
            };

            let store_op = if use_store_op {
                render_pass_create_info.store_op[index]
            } else {
                ash::vk::AttachmentStoreOp::STORE
            };

            let final_layout = if use_final_layouts {
                render_pass_create_info.final_layouts[index]
            } else if depth {
                    depth_attachment_count += 1;
                    ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
            } else {
                    ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            };

            // Description
            let description = ash::vk::AttachmentDescription {
                flags: Default::default(),
                format: *format,
                samples: ash::vk::SampleCountFlags::TYPE_1,
                load_op,
                store_op,
                stencil_load_op: ash::vk::AttachmentLoadOp::DONT_CARE, // TODO
                stencil_store_op: ash::vk::AttachmentStoreOp::DONT_CARE, // TODO
                initial_layout: ash::vk::ImageLayout::UNDEFINED,
                final_layout,
            };

            descriptions.push(description);

            // Reference

            if depth {
                depth_reference = ash::vk::AttachmentReference {
                    attachment: index as u32,
                    layout: final_layout,
                };
            } else {
                references.push(ash::vk::AttachmentReference {
                    attachment: index as u32,
                    layout: final_layout,
                });
            }
        }

        let mut subpass = *ash::vk::SubpassDescription::builder()
            .pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&references);
        if depth_attachment_count > 0 {
            subpass.p_depth_stencil_attachment = &depth_reference;
        }

        let subpass = [subpass];

        let render_pass_create_info_vk = ash::vk::RenderPassCreateInfo::builder()
            .attachments(&descriptions)
            .subpasses(&subpass)
            .dependencies(&render_pass_create_info.dependencies);

        unsafe { device.get_device().create_render_pass(&render_pass_create_info_vk, None)}.unwrap_or_else(|e| {
            log::error!("Failed to create render pass, error: {e}");
            panic!();
        })
    }

    pub(crate) fn from(
        device: Arc<VkDevice>,
        image: Arc<VkImage>,
        render_pass: ash::vk::RenderPass,
        extent: ash::vk::Extent2D,
    ) -> Self {
        let image_views = [image.get_view(0)];
        let create_info = ash::vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&image_views)
            .attachment_count(1)
            .width(extent.width)
            .height(extent.height)
            .layers(1);

        let handle = unsafe { device.get_device().create_framebuffer(&create_info, None) }
            .unwrap_or_else(|e| {
                log::error!("Failed to create framebuffer, error: {e}");
                panic!();
            });

        let final_layouts = vec![ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL];

        Self {
            images: vec![image],
            render_pass,
            extent,
            handle,
            device,
            attachment_formats: Vec::new(),
            drop_render_pass: false,
            final_layouts,
        }
    }

    pub(crate) fn get_render_pass(&self) -> ash::vk::RenderPass {
        self.render_pass
    }

    pub(crate) fn begin_render_pass(
        &self,
        command_buffer: &VkCommandBuffer,
        clear_colors: &[ClearColor],
        extent: ash::vk::Extent2D,
    ) {
        let viewport = [ash::vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = ash::vk::Rect2D {
            offset: ash::vk::Offset2D { x: 0, y: 0 },
            extent,
        };

        unsafe {
            self.device
                .get_device()
                .cmd_set_viewport(command_buffer.get_handle(), 0, &viewport)
        };
        unsafe {
            self.device
                .get_device()
                .cmd_set_scissor(command_buffer.get_handle(), 0, &[scissors])
        };

        let clear_values = clear_colors
            .iter()
            .map(|color| ash::vk::ClearValue::from(*color))
            .collect::<Vec<_>>();

        let begin_info = ash::vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.handle)
            .render_area(scissors)
            .clear_values(&clear_values);

        unsafe {
            self.device.get_device().cmd_begin_render_pass(
                command_buffer.get_handle(),
                &begin_info,
                ash::vk::SubpassContents::INLINE,
            )
        };

        for (index, layout) in self.final_layouts.iter().enumerate() {
            self.images[index].set_layout(*layout);
        }
    }

    pub(crate) fn end_render_pass(&self, command_buffer: &VkCommandBuffer) {
        unsafe {
            self.device
                .get_device()
                .cmd_end_render_pass(command_buffer.get_handle())
        };
    }

    pub(crate) fn get_image(&self, index: u32) -> Arc<VkImage> {
        self.images[index as usize].clone()
    }
}

impl Drop for VkFramebuffer {
    fn drop(&mut self) {
        unsafe {
            if self.drop_render_pass {
                self.device
                    .get_device()
                    .destroy_render_pass(self.render_pass, None);
            }
            self.device
                .get_device()
                .destroy_framebuffer(self.handle, None);
            for image in &self.images {
                for image_view in image.get_views() {
                    self.device
                        .get_device()
                        .destroy_image_view(*image_view, None);
                }
            }
        };
    }
}

unsafe impl Send for VkFramebuffer {}
unsafe impl Sync for VkFramebuffer {}
