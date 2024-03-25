use crate::render::backend::framebuffer::{ClearColor, MVFramebufferCreateInfo};
use crate::render::backend::vulkan::command_buffer::VkCommandBuffer;
use crate::render::backend::vulkan::device::VkDevice;
use std::sync::Arc;

pub(crate) struct CreateInfo {}

impl From<MVFramebufferCreateInfo> for CreateInfo {
    fn from(value: MVFramebufferCreateInfo) -> Self {
        CreateInfo {}
    }
}

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
    images: Vec<ash::vk::Image>,
    image_views: Vec<ash::vk::ImageView>,
    handle: ash::vk::Framebuffer,
    render_pass: ash::vk::RenderPass,

    extent: ash::vk::Extent2D,
    attachment_formats: Vec<ash::vk::Format>,
    drop_render_pass: bool,
    //final_layout: Vec<ash::vk::ImageLa> we don't need it for now
}

impl VkFramebuffer {
    pub(crate) fn new(device: Arc<VkDevice>, info: CreateInfo) -> Self {
        todo!()
    }

    pub(crate) fn from(
        device: Arc<VkDevice>,
        image: ash::vk::Image,
        image_view: ash::vk::ImageView,
        render_pass: ash::vk::RenderPass,
        extent: ash::vk::Extent2D,
    ) -> Self {
        let create_info = ash::vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&[image_view])
            .attachment_count(1)
            .width(extent.width)
            .height(extent.height)
            .layers(1)
            .build();

        let handle = unsafe { device.get_device().create_framebuffer(&create_info, None) }
            .unwrap_or_else(|e| {
                log::error!("Failed to create framebuffer, error: {e}");
                panic!();
            });

        Self {
            images: vec![image],
            image_views: vec![image_view],
            render_pass,
            extent,
            handle,
            device,
            attachment_formats: Vec::new(),
            drop_render_pass: false,
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
        let viewport = [ash::vk::Viewport::builder()
            .x(0.0f32)
            .y(0.0f32)
            .width(extent.width as f32)
            .height(extent.height as f32)
            .min_depth(0.0f32)
            .max_depth(0.0f32)
            .build()];

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

        let begin_info = &std::hint::black_box(ash::vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.handle)
            .render_area(scissors)
            .clear_values(&clear_values)
            .build());

        let ptr = unsafe {
            let ptr = begin_info as *const _ as *const u8;
            std::slice::from_raw_parts(ptr, 64)
        };

        unsafe {
            self.device.get_device().cmd_begin_render_pass(
                command_buffer.get_handle(),
                begin_info,
                ash::vk::SubpassContents::INLINE,
            )
        };
    }

    pub(crate) fn end_render_pass(&self, command_buffer: &VkCommandBuffer) {
        unsafe {
            self.device
                .get_device()
                .cmd_end_render_pass(command_buffer.get_handle())
        };
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
            for image_view in &self.image_views {
                self.device
                    .get_device()
                    .destroy_image_view(*image_view, None);
            }
        };
    }
}

unsafe impl Send for VkFramebuffer {}
unsafe impl Sync for VkFramebuffer {}
