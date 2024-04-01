use crate::render::backend::device::Device;
use crate::render::backend::vulkan::sampler::VkSampler;
use mvcore_proc_macro::graphics_item;

#[derive(Copy, Clone)]
pub enum SamplerAddressMode {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
}

#[derive(Copy, Clone)]
pub enum Filter {
    Nearest,
    Linear,
}

#[derive(Copy, Clone)]
pub enum MipmapMode {
    Nearest,
    Linear,
}

pub struct MVSamplerCreateInfo {
    pub address_mode: SamplerAddressMode,
    pub filter_mode: Filter,
    pub mipmap_mode: MipmapMode,
    pub anisotropy: bool,

    pub label: Option<String>,
}

#[graphics_item(ref)]
pub enum Sampler {
    Vulkan(VkSampler),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl Sampler {
    pub fn new(device: Device, create_info: MVSamplerCreateInfo) -> Self {
        match device {
            Device::Vulkan(device) => Sampler::Vulkan(VkSampler::new(device, create_info.into())),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }
}
