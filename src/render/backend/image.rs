use crate::render::backend::buffer::{Buffer, MemoryProperties};
use crate::render::backend::command_buffer::CommandBuffer;
use crate::render::backend::device::Device;
use crate::render::backend::vulkan::image::VkImage;
use crate::render::backend::Extent2D;
use bitflags::bitflags;
use mvcore_proc_macro::graphics_item;
use std::ffi::CString;
use std::sync::Arc;

pub(crate) enum ImageType {
    Image2D,
    Image2DArray,
    Cubemap,
}

pub(crate) enum ImageTiling {
    Optimal,
    Linear,
}

pub(crate) enum ImageFormat {
    R8,
    R8G8,
    R8G8B8,
    R8G8B8A8,
    R32,
    R32G32,
    R32G32B32,
    R32G32B32A32,
    D16,
    D16S8,
    D24,
    D32,
}

pub(crate) enum ImageLayout {
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
    pub(crate) struct ImageUsage: u8 {
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
    pub(crate) struct ImageAspect: u8 {
        const COLOR = 1 << 0;
        const DEPTH = 1 << 1;
        const STECIL = 1 << 2;
        const METADATA = 1 << 3;
    }
}

pub(crate) struct MVImageCreateInfo {
    pub(crate) size: Extent2D,
    pub(crate) format: ImageFormat,
    pub(crate) usage: ImageUsage,
    pub(crate) memory_properties: MemoryProperties,
    pub(crate) aspect: ImageAspect,
    pub(crate) tiling: ImageTiling,
    pub(crate) layer_count: u32,
    pub(crate) image_type: ImageType,
    pub(crate) cubemap: bool,
    pub(crate) memory_usage_flags: gpu_alloc::UsageFlags,
    pub(crate) data: Option<Vec<u8>>,

    pub(crate) label: Option<String>,
}

#[graphics_item(clone)]
#[derive(Clone)]
pub(crate) enum Image {
    Vulkan(Arc<VkImage>),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl Image {
    pub(crate) fn new(device: Device, create_info: MVImageCreateInfo) -> Self {
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

    pub(crate) fn transition_layout(
        &self,
        new_layout: ImageLayout,
        command_buffer: Option<&CommandBuffer>,
        src: ash::vk::AccessFlags,
        dst: ash::vk::AccessFlags,
    ) {
        match self {
            Image::Vulkan(image) => image.transition_layout(
                new_layout.into(),
                command_buffer.map(|cmd| cmd.as_vulkan()),
                src,
                dst,
            ),
            #[cfg(target_os = "macos")]
            Image::Metal => unreachable!(),
            #[cfg(target_os = "windows")]
            Image::DirectX => unreachable!(),
        }
    }

    pub(crate) fn copy_buffer_to_image(
        &self,
        buffer: &Buffer,
        command_buffer: Option<&CommandBuffer>,
    ) {
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

    pub(crate) fn get_extent(&self) -> Extent2D {
        match self {
            Image::Vulkan(image) => image.get_extent().into(),
            #[cfg(target_os = "macos")]
            Image::Metal => unreachable!(),
            #[cfg(target_os = "windows")]
            Image::DirectX => unreachable!(),
        }
    }
}
