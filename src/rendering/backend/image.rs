use std::sync::Arc;

use bitflags::bitflags;
use image::ColorType;

use mvengine_proc_macro::graphics_item;
use crate::rendering::backend::buffer::{Buffer, MemoryProperties};
use crate::rendering::backend::command_buffer::CommandBuffer;
use crate::rendering::backend::device::Device;
use crate::rendering::backend::vulkan::image::VkImage;
use crate::rendering::backend::Extent2D;

pub enum ImageType {
    Image2D,
    Image2DArray,
    Cubemap,
}

pub enum ImageTiling {
    Optimal,
    Linear,
}

pub enum ImageFormat {
    R8,
    R8G8,
    R8G8B8,
    R8G8B8A8,
    R16,
    R16G16,
    R16G16B16,
    R16G16B16A16,
    R32,
    R32G32,
    R32G32B32,
    R32G32B32A32,
    D16,
    D16S8,
    D24,
    D32,
}

impl From<ColorType> for ImageFormat {
    fn from(value: ColorType) -> Self {
        match value {
            ColorType::L8 => ImageFormat::R8,
            ColorType::La8 => ImageFormat::R8G8,
            ColorType::Rgb8 => ImageFormat::R8G8B8,
            ColorType::Rgba8 => ImageFormat::R8G8B8A8,
            ColorType::L16 => ImageFormat::R16,
            ColorType::La16 => ImageFormat::R16G16,
            ColorType::Rgb16 => ImageFormat::R16G16B16,
            ColorType::Rgba16 => ImageFormat::R16G16B16A16,
            ColorType::Rgb32F => ImageFormat::R32G32B32,
            ColorType::Rgba32F => ImageFormat::R32G32B32A32,
            _ => unimplemented!(),
        }
    }
}

pub enum ImageLayout {
    Undefined,
    General,
    ColorAttachmentOptimal,
    DepthStencilAttachmentOptimal,
    DepthStencilReadOnlyOptimal,
    ShaderReadOnlyOptimal,
    TransferSrcOptimal,
    TransferDstOptimal,
    Preinitialized,
    PresentSrc,
}

bitflags! {
    pub struct ImageUsage: u8 {
        const TRANSFER_SRC = 1 << 0;
        const TRANSFER_DST = 1 << 1;
        const SAMPLED = 1 << 2;
        const STORAGE = 1 << 3;
        const COLOR_ATTACHMENT = 1 << 4;
        const DEPTH_STENCIL_ATTACHMENT = 1 << 5;
        const TRANSIENT_ATTACHMENT = 1 << 6;
        const INPUT_ATTACHMENT = 1 << 7;
    }
}
bitflags! {
    pub struct ImageAspect: u8 {
        const COLOR = 1 << 0;
        const DEPTH = 1 << 1;
        const STECIL = 1 << 2;
        const METADATA = 1 << 3;
    }
}

pub struct MVImageCreateInfo {
    pub size: Extent2D,
    pub format: ImageFormat,
    pub usage: ImageUsage,
    pub memory_properties: MemoryProperties,
    pub aspect: ImageAspect,
    pub tiling: ImageTiling,
    pub layer_count: u32,
    pub image_type: ImageType,
    pub cubemap: bool,
    pub memory_usage_flags: gpu_alloc::UsageFlags,
    pub data: Option<Vec<u8>>,

    pub label: Option<String>,
}

#[graphics_item(clone)]
#[derive(Clone)]
pub enum Image {
    Vulkan(Arc<VkImage>),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl Image {
    pub fn new(device: Device, create_info: MVImageCreateInfo) -> Self {
        match device {
            Device::Vulkan(device) => {
                Image::Vulkan(VkImage::new(device, create_info.into()).into())
            }
            #[cfg(target_os = "macos")]
            Device::Metal => unreachable!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unreachable!(),
        }
    }

    pub fn transition_layout(
        &self,
        new_layout: ImageLayout,
        command_buffer: Option<&CommandBuffer>,
        src: AccessFlags,
        dst: AccessFlags,
    ) {
        match self {
            Image::Vulkan(image) => image.transition_layout(
                new_layout.into(),
                command_buffer.map(|cmd| cmd.as_vulkan()),
                ash::vk::AccessFlags::from_raw(src.bits()),
                ash::vk::AccessFlags::from_raw(dst.bits()),
            ),
            #[cfg(target_os = "macos")]
            Image::Metal => unreachable!(),
            #[cfg(target_os = "windows")]
            Image::DirectX => unreachable!(),
        }
    }

    pub fn copy_buffer_to_image(&self, buffer: &Buffer, command_buffer: Option<&CommandBuffer>) {
        match self {
            Image::Vulkan(image) => image.copy_buffer_to_image(
                buffer.as_vulkan(),
                command_buffer.map(|cmd| cmd.as_vulkan()),
            ),
            #[cfg(target_os = "macos")]
            Image::Metal => unreachable!(),
            #[cfg(target_os = "windows")]
            Image::DirectX => unreachable!(),
        }
    }

    pub fn get_extent(&self) -> Extent2D {
        match self {
            Image::Vulkan(image) => image.get_extent().into(),
            #[cfg(target_os = "macos")]
            Image::Metal => unreachable!(),
            #[cfg(target_os = "windows")]
            Image::DirectX => unreachable!(),
        }
    }
}

bitflags! {
    pub struct AccessFlags: u32 {
        const INDIRECT_COMMAND_READ = 1 << 0;
        const INDEX_READ = 1 << 1;
        const VERTEX_ATTRIBUTE_READ = 1 << 2;
        const UNIFORM_READ = 1 << 3;
        const INPUT_ATTACHMENT_READ = 1 << 4;
        const SHADER_WRITE = 1 << 6;
        const COLOR_ATTACHMENT_READ = 1 << 7;
        const COLOR_ATTACHMENT_WRITE = 1 << 8;
        const DEPTH_STENCIL_ATTACHMENT_READ = 1 << 9;
        const DEPTH_STENCIL_ATTACHMENT_WRITE = 1 << 10;
        const TRANSFER_READ = 1 << 11;
        const TRANSFER_WRITE = 1 << 12;
        const HOST_READ = 1 << 13;
        const HOST_WRITE = 1 << 14;
        const MEMORY_READ = 1 << 15;
        const MEMORY_WRITE = 1 << 16;
        #[cfg(feature = "ray-tracing")]
        const ACCELERATION_STRUCTURE_READ_KHR = 1 << 21;
        #[cfg(feature = "ray-tracing")]
        const ACCELERATION_STRUCTURE_WRITE_KHR = 1 << 22;
    }
}
