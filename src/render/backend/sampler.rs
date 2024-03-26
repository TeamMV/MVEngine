use mvcore_proc_macro::graphics_item;
use crate::render::backend::device::Device;
use crate::render::backend::vulkan::sampler::VkSampler;

#[derive(Copy, Clone)]
pub(crate) enum SamplerAddressMode {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
}

#[derive(Copy, Clone)]
pub(crate) enum Filter {
    Nearest,
    Linear
}

#[derive(Copy, Clone)]
pub(crate) enum MipmapMode {
    Nearest,
    Linear,
}

pub(crate) struct MVSamplerCreateInfo {
    pub(crate) address_mode: SamplerAddressMode,
    pub(crate) filter_mode: Filter,
    pub(crate) mipmap_mode: MipmapMode,

    pub(crate) label: Option<String>,
}

#[graphics_item(ref)]
pub(crate) enum Sampler {
    Vulkan(VkSampler),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl Sampler {
    pub(crate) fn new(device: Device, create_info: MVSamplerCreateInfo) -> Self {
        match device {
            Device::Vulkan(device) => Sampler::Vulkan(VkSampler::new(device, create_info.into()).into()),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

}