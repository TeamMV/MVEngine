use crate::rendering::backend::command_buffer::{CommandBufferLevel, MVCommandBufferCreateInfo};
use crate::rendering::backend::vulkan::buffer::VkBuffer;
use crate::rendering::backend::vulkan::device::VkDevice;
use crate::rendering::backend::vulkan::image::VkImage;
use ash::vk::{AccessFlags, CommandBufferUsageFlags, ImageLayout};
use std::sync::Arc;

pub(crate) struct CreateInfo {
    level: ash::vk::CommandBufferLevel,
    pool: ash::vk::CommandPool,

    #[cfg(debug_assertions)]
    debug_name: std::ffi::CString,
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
            debug_name: crate::rendering::backend::to_ascii_cstring(
                value.label.unwrap_or_default(),
            ),
        }
    }
}

pub struct VkCommandBuffer {
    pub(crate) device: Arc<VkDevice>,
    pub(crate) handle: ash::vk::CommandBuffer,
}

impl VkCommandBuffer {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: CreateInfo) -> Self {
        let vk_create_info = ash::vk::CommandBufferAllocateInfo::builder()
            .level(create_info.level)
            .command_pool(create_info.pool)
            .command_buffer_count(1);

        let buffer = unsafe {
            device
                .get_device()
                .allocate_command_buffers(&vk_create_info)
        }
        .unwrap_or_else(|e| {
            log::error!("Failed to allocate command buffer, error: {e}");
            panic!("Critical Vulkan driver ERROR")
        })[0];

        #[cfg(debug_assertions)]
        device.set_object_name(
            &ash::vk::ObjectType::COMMAND_BUFFER,
            ash::vk::Handle::as_raw(buffer),
            create_info.debug_name.as_c_str(),
        );

        Self {
            device,
            handle: buffer,
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
            .flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device
                .get_device()
                .begin_command_buffer(self.handle, &begin_info)
        }
        .unwrap_or_else(|e| {
            log::error!("Failed to begin recording command buffer, error: {e}");
            panic!("Critical Vulkan driver ERROR")
        })
    }

    pub(crate) fn end(&self) {
        unsafe { self.device.get_device().end_command_buffer(self.handle) }.unwrap_or_else(|e| {
            log::error!("Failed to end recording command buffer, error: {e}");
            panic!("Critical Vulkan driver ERROR")
        })
    }

    pub(crate) fn draw(
        &self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        unsafe {
            self.device.get_device().cmd_draw(
                self.handle,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
        };
    }

    pub(crate) fn draw_indexed(
        &self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        first_instance: u32,
    ) {
        unsafe {
            self.device.get_device().cmd_draw_indexed(
                self.handle,
                index_count,
                instance_count,
                first_index,
                0,
                first_instance,
            )
        };
    }

    pub(crate) fn bind_vertex_buffer(&self, buffer: &VkBuffer) {
        unsafe {
            self.device.get_device().cmd_bind_vertex_buffers(
                self.handle,
                0,
                &[buffer.get_buffer()],
                &[0],
            )
        }
    }

    pub(crate) fn bind_index_buffer(&self, buffer: &VkBuffer) {
        unsafe {
            self.device.get_device().cmd_bind_index_buffer(
                self.handle,
                buffer.get_buffer(),
                0,
                ash::vk::IndexType::UINT32,
            )
        };
    }

    pub(crate) fn blit_image(&self, src_image: Arc<VkImage>, dst_image: Arc<VkImage>) {
        src_image.transition_layout(
            ImageLayout::TRANSFER_SRC_OPTIMAL,
            Some(self),
            AccessFlags::empty(),
            AccessFlags::empty(),
        );
        dst_image.transition_layout(
            ImageLayout::TRANSFER_DST_OPTIMAL,
            Some(self),
            AccessFlags::empty(),
            AccessFlags::empty(),
        );

        let blit = ash::vk::ImageBlit {
            src_subresource: ash::vk::ImageSubresourceLayers {
                aspect_mask: src_image.aspect,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: src_image.layer_count,
            },
            src_offsets: [
                ash::vk::Offset3D { x: 0, y: 0, z: 0 },
                ash::vk::Offset3D {
                    x: src_image.get_extent().width as i32,
                    y: src_image.get_extent().height as i32,
                    z: 1,
                },
            ],
            dst_subresource: ash::vk::ImageSubresourceLayers {
                aspect_mask: dst_image.aspect,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: dst_image.layer_count,
            },
            dst_offsets: [
                ash::vk::Offset3D { x: 0, y: 0, z: 0 },
                ash::vk::Offset3D {
                    x: dst_image.get_extent().width as i32,
                    y: dst_image.get_extent().height as i32,
                    z: 1,
                },
            ],
        };

        unsafe {
            self.device.get_device().cmd_blit_image(
                self.handle,
                src_image.handle,
                src_image.layout.get_val(),
                dst_image.handle,
                dst_image.layout.get_val(),
                &[blit],
                ash::vk::Filter::LINEAR,
            );
        }
    }

    pub(crate) fn dispatch(&self, extent: ash::vk::Extent3D) {
        unsafe {
            self.device.get_device().cmd_dispatch(
                self.handle,
                extent.width,
                extent.height,
                extent.depth,
            )
        };
    }

    pub(crate) fn get_handle(&self) -> ash::vk::CommandBuffer {
        self.handle
    }
}
