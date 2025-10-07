use ash::vk;
use gpu_alloc::AllocationError;
use image::ImageError;

pub enum RenderingError {
    ShaderError(shaderc::Error),
    AllocationError(AllocationError),
    VulkanError(vk::Result),
    ImageError(ImageError),
    Other(String)
}