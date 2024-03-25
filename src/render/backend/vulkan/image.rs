use std::ffi::CString;
use std::fmt::format;
use std::sync::Arc;
use ash::vk::Handle;
use crate::render::backend::image::{ImageFormat, ImageLayout, ImageTiling, ImageType, MVImageCreateInfo};
use crate::render::backend::to_ascii_cstring;
use crate::render::backend::vulkan::buffer;
use crate::render::backend::vulkan::buffer::VkBuffer;
use crate::render::backend::vulkan::command_buffer::VkCommandBuffer;
use crate::render::backend::vulkan::device::VkDevice;

pub(crate) struct VkImage {
    device: Arc<VkDevice>,

    handle: ash::vk::Image,
    image_views: Vec<ash::vk::ImageView>,
    format: ash::vk::Format,
    aspect: ash::vk::ImageAspectFlags,
    tiling: ash::vk::ImageTiling,
    layer_count: u32,
    image_type: ash::vk::ImageType,
    size: ash::vk::Extent2D,
    mip_level_count: u32,
    usage: ash::vk::ImageUsageFlags,
    memory_properties: ash::vk::MemoryPropertyFlags,
    layout: ash::vk::ImageLayout,
    memory_usage_flags: gpu_alloc::UsageFlags
}

pub(crate) struct CreateInfo {
    pub size: ash::vk::Extent2D,
    pub format: ash::vk::Format,
    pub usage: ash::vk::ImageUsageFlags,
    pub memory_properties: ash::vk::MemoryPropertyFlags,
    pub aspect: ash::vk::ImageAspectFlags,
    pub tiling: ash::vk::ImageTiling,
    pub layer_count: u32,
    pub image_type: ImageType,
    pub cubemap: bool,
    pub memory_usage_flags: gpu_alloc::UsageFlags,
    pub data: Option<Vec<u8>>,

    #[cfg(debug_assertions)]
    pub debug_name: CString
}

impl From<MVImageCreateInfo> for CreateInfo {
    fn from(value: MVImageCreateInfo) -> Self {
        CreateInfo {
            size: value.size.into(),
            format: value.format.into(),
            usage: ash::vk::ImageUsageFlags::from_raw(value.usage.bits() as u32),
            memory_properties: ash::vk::MemoryPropertyFlags::from_raw(value.memory_properties.bits() as u32),
            aspect: ash::vk::ImageAspectFlags::from_raw(value.aspect.bits() as u32),
            tiling: match value.tiling {
                ImageTiling::Optimal => ash::vk::ImageTiling::OPTIMAL,
                ImageTiling::Linear => ash::vk::ImageTiling::LINEAR,
            },
            layer_count: value.layer_count,
            image_type: value.image_type,
            cubemap: value.cubemap,
            memory_usage_flags: value.memory_usage_flags,
            data: value.data,

            #[cfg(debug_assertions)]
            debug_name: to_ascii_cstring(value.label.unwrap_or_default()),
        }
    }
}

impl From<ImageLayout> for ash::vk::ImageLayout {
    fn from(value: ImageLayout) -> Self {
        match value {
            ImageLayout::Undefined => ash::vk::ImageLayout::UNDEFINED,
            ImageLayout::General => ash::vk::ImageLayout::GENERAL,
            ImageLayout::ColorAttachmentOptimal => ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ImageLayout::DepthStencilAttachmentOptimal => ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ImageLayout::DepthStencilReadOnlyOptimal => ash::vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
            ImageLayout::ShaderReadOnlyOptimal => ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ImageLayout::TransferSrcOptimal => ash::vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            ImageLayout::TransferDstOptimal => ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            ImageLayout::Preinitialized => ash::vk::ImageLayout::PREINITIALIZED,
        }
    }
}

impl From<ImageFormat> for ash::vk::Format {
    fn from(value: ImageFormat) -> Self {
        match value {
            ImageFormat::R8 => ash::vk::Format::R8_UNORM,
            ImageFormat::R8G8 => ash::vk::Format::R8G8_UNORM,
            ImageFormat::R8G8B8 => ash::vk::Format::R8G8B8_UNORM,
            ImageFormat::R8G8B8A8 => ash::vk::Format::R8G8B8A8_UNORM,
            ImageFormat::R32 => ash::vk::Format::R32_SFLOAT,
            ImageFormat::R32B32 => ash::vk::Format::R32G32_SFLOAT,
            ImageFormat::R32B32G32 => ash::vk::Format::R32G32B32_SFLOAT,
            ImageFormat::R32B32G32A32 => ash::vk::Format::R32G32B32A32_SFLOAT,
        }
    }
}

impl VkImage {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: CreateInfo) -> Self {
        let flags = if create_info.cubemap {
            ash::vk::ImageCreateFlags::CUBE_COMPATIBLE
        } else {
            ash::vk::ImageCreateFlags::empty()
        };

        let create_info_vk = ash::vk::ImageCreateInfo::builder()
            .image_type(ash::vk::ImageType::TYPE_2D)
            .extent(ash::vk::Extent3D{width: create_info.size.width, height: create_info.size.height, depth: 1})
            .mip_levels(1)
            .array_layers(create_info.layer_count)
            .format(create_info.format)
            .tiling(create_info.tiling)
            .initial_layout(ash::vk::ImageLayout::UNDEFINED)
            .usage(create_info.usage)
            .samples(ash::vk::SampleCountFlags::TYPE_1)
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
            .flags(flags);

        let (image, block) = device.allocate_image(&create_info_vk, create_info.memory_properties, create_info.memory_usage_flags);


        #[cfg(debug_assertions)]
        device.set_object_name(&ash::vk::ObjectType::IMAGE, image.as_raw(), create_info.debug_name.as_c_str());

        let view_type = match create_info.image_type {
            ImageType::Image2D => { ash::vk::ImageViewType::TYPE_2D }
            ImageType::Image2DArray => { ash::vk::ImageViewType::TYPE_2D_ARRAY }
            ImageType::Cubemap => { ash::vk::ImageViewType::CUBE_ARRAY }
        };
        let mut views = Vec::new();
        for i in 0..create_info.layer_count {
            let view_info = ash::vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(view_type)
                .format(create_info.format)
                .subresource_range(ash::vk::ImageSubresourceRange{
                    aspect_mask: create_info.aspect,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: create_info.layer_count,
                });

            let view = unsafe { device.get_device().create_image_view(&view_info, None) }.unwrap_or_else(|e| {
                log::error!("Failed to create image view, error: {e}");
                panic!();
            });

            views.push(view);

            #[cfg(debug_assertions)]
            device.set_object_name(&ash::vk::ObjectType::IMAGE_VIEW, view.as_raw(), create_info.debug_name.as_c_str());
        }

        let this = Self {
            device: device.clone(),
            handle: image,
            image_views: views,
            format: create_info.format,
            aspect: create_info.aspect,
            tiling: create_info.tiling,
            layer_count: create_info.layer_count,
            image_type: ash::vk::ImageType::TYPE_2D,
            size: create_info.size,
            mip_level_count: 1,
            usage: create_info.usage,
            memory_properties: create_info.memory_properties,
            layout: ash::vk::ImageLayout::UNDEFINED,
            memory_usage_flags: create_info.memory_usage_flags,
        };

        if let Some(data) = create_info.data {
            let size = (Self::format_to_size(create_info.format) * create_info.size.height * create_info.size.width) as ash::vk::DeviceSize;
            let buffer_create_info = buffer::CreateInfo {
                instance_size: size,
                instance_count: 1,
                minimum_alignment: 1,
                usage_flags: ash::vk::BufferUsageFlags::TRANSFER_SRC,
                memory_properties: ash::vk::MemoryPropertyFlags::HOST_VISIBLE | ash::vk::MemoryPropertyFlags::HOST_COHERENT,
                memory_usage_flags: gpu_alloc::UsageFlags::TRANSIENT | gpu_alloc::UsageFlags::HOST_ACCESS,

                #[cfg(debug_assertions)]
                debug_name: CString::new("Image staging buffer").unwrap()
            };

            let mut buffer = VkBuffer::new(device.clone(), buffer_create_info);
            buffer.map();
            buffer.write_to_buffer(&data, 0, None);
            buffer.unmap();

            this.transition_layout(ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL, None);
            this.copy_buffer_to_image(&buffer, None);
            this.transition_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, None);
        }

        this
    }

    #[allow(clippy::identity_op)]
    fn format_to_size(format: ash::vk::Format) -> u32 {
        match format {
            ash::vk::Format::R32G32B32A32_SFLOAT => {
                4 * 4
            }
            ash::vk::Format::R32G32B32_SFLOAT => {
                4 * 3
            }
            ash::vk::Format::R32G32_SFLOAT => {
                4 * 2
            }
            ash::vk::Format::R32_SFLOAT => {
                4 * 1
            }
            ash::vk::Format::R8G8B8A8_UNORM => {
                1 * 4
            }
            ash::vk::Format::R8G8B8_UNORM => {
                1 * 3
            }
            ash::vk::Format::R8G8_UNORM => {
                1 * 2
            }
            ash::vk::Format::R8_UNORM => {
                1 * 1
            }
            _ => {
                log::error!("Format unsupported!");
                panic!();
            }
        }
    }

    pub(crate) fn transition_layout(&self, new_layout: ash::vk::ImageLayout, provided_cmd: Option<&VkCommandBuffer>) {
        let (cmd, end) = if let Some(cmd) = provided_cmd {
            (cmd.get_handle(), false)
        } else {
            (
                self.device
                    .begin_single_time_command(self.device.get_graphics_command_pool()),
                true,
            )
        };

        let mut src_access = ash::vk::AccessFlags::empty();
        let mut dst_access = ash::vk::AccessFlags::empty();
        let mut src_stage = ash::vk::PipelineStageFlags::empty();
        let mut dst_stage = ash::vk::PipelineStageFlags::empty();
        
        match self.layout {
            ash::vk::ImageLayout::GENERAL => {
                src_access |= ash::vk::AccessFlags::SHADER_WRITE | ash::vk::AccessFlags::SHADER_READ;
                src_stage |= ash::vk::PipelineStageFlags::COMPUTE_SHADER;

                #[cfg(feature = "ray-tracing")]
                { src_stage |= ash::vk::PipelineStageFlags::RAY_TRACING_SHADER_KHR; }
            }
            ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL => {
                src_access |= ash::vk::AccessFlags::SHADER_READ;
                src_stage |= ash::vk::PipelineStageFlags::FRAGMENT_SHADER | ash::vk::PipelineStageFlags::COMPUTE_SHADER;

                #[cfg(feature = "ray-tracing")]
                { src_stage |= ash::vk::PipelineStageFlags::RAY_TRACING_SHADER_KHR; }
            }
            ash::vk::ImageLayout::TRANSFER_SRC_OPTIMAL => {
                src_access |= ash::vk::AccessFlags::TRANSFER_READ;
                src_stage |= ash::vk::PipelineStageFlags::TRANSFER;
            }
            ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL => {
                src_access |= ash::vk::AccessFlags::TRANSFER_WRITE;
                src_stage |= ash::vk::PipelineStageFlags::TRANSFER;
            }
            _ => {}
        }

        match new_layout {
            ash::vk::ImageLayout::GENERAL => {
                dst_access |= ash::vk::AccessFlags::SHADER_WRITE | ash::vk::AccessFlags::SHADER_READ;
                dst_stage |= ash::vk::PipelineStageFlags::COMPUTE_SHADER;

                #[cfg(feature = "ray-tracing")]
                { dst_stage |= ash::vk::PipelineStageFlags::RAY_TRACING_SHADER_KHR; }
            }
            ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL => {
                dst_access |= ash::vk::AccessFlags::SHADER_READ;
                dst_stage |= ash::vk::PipelineStageFlags::FRAGMENT_SHADER | ash::vk::PipelineStageFlags::COMPUTE_SHADER;

                #[cfg(feature = "ray-tracing")]
                { dst_stage |= ash::vk::PipelineStageFlags::RAY_TRACING_SHADER_KHR; }
            }
            ash::vk::ImageLayout::TRANSFER_SRC_OPTIMAL => {
                dst_access |= ash::vk::AccessFlags::TRANSFER_READ;
                dst_stage |= ash::vk::PipelineStageFlags::TRANSFER;
            }
            ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL => {
                src_access |= ash::vk::AccessFlags::TRANSFER_WRITE;
                dst_stage |= ash::vk::PipelineStageFlags::TRANSFER;
            }
            _ => {}
        }

        let subresource_range = ash::vk::ImageSubresourceRange{
            aspect_mask: self.aspect,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: self.layer_count,
        };

        let barrier = ash::vk::ImageMemoryBarrier::builder()
            .old_layout(self.layout)
            .new_layout(new_layout)
            .src_queue_family_index(ash::vk::QUEUE_FAMILY_EXTERNAL)
            .dst_queue_family_index(ash::vk::QUEUE_FAMILY_EXTERNAL)
            .image(self.handle)
            .subresource_range(subresource_range)
            .src_access_mask(src_access)
            .dst_access_mask(dst_access);

        let barriers = [*barrier];


        unsafe { self.device.get_device().cmd_pipeline_barrier(
            cmd,
            src_stage,
            dst_stage,
            ash::vk::DependencyFlags::empty(),
            &[],
            &[],
            &barriers)
        }

        if end {
            self.device.end_single_time_command(
                cmd,
                self.device.get_graphics_command_pool(),
                self.device.get_graphics_queue(),
            );
        }
    }

    pub(crate) fn copy_buffer_to_image(&self, buffer: &VkBuffer, provided_cmd: Option<&VkCommandBuffer>) {
        let (cmd, end) = if let Some(cmd) = provided_cmd {
            (cmd.get_handle(), false)
        } else {
            (
                self.device
                    .begin_single_time_command(self.device.get_graphics_command_pool()),
                true,
            )
        };

        let subresource_range = ash::vk::ImageSubresourceLayers{
            aspect_mask: self.aspect,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: self.layer_count,
        };

        let copy_region = ash::vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: subresource_range,
            image_offset: ash::vk::Offset3D { x: 0, y: 0, z : 0},
            image_extent: ash::vk::Extent3D { width: self.size.width, height: self.size.height, depth: 1}
        };

        unsafe { self.device.get_device().cmd_copy_buffer_to_image(cmd, buffer.get_buffer(), self.handle, self.layout, &[copy_region])};

        if end {
            self.device.end_single_time_command(
                cmd,
                self.device.get_graphics_command_pool(),
                self.device.get_graphics_queue(),
            );
        }
    }

    pub(crate) fn get_view(&self) -> ash::vk::ImageView {
        self.image_views[0]
    }

    pub(crate) fn get_handle(&self) -> ash::vk::Image {
        self.handle
    }

}