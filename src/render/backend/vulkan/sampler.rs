use crate::render::backend::sampler::{
    Filter, MVSamplerCreateInfo, MipmapMode, SamplerAddressMode,
};
use crate::render::backend::to_ascii_cstring;
use crate::render::backend::vulkan::device::VkDevice;
use std::ffi::CString;
use std::sync::Arc;

pub(crate) struct VkSampler {
    device: Arc<VkDevice>,
    handle: ash::vk::Sampler,
}

pub(crate) struct CreateInfo {
    address_mode: ash::vk::SamplerAddressMode,
    filter_mode: ash::vk::Filter,
    mipmap_mode: ash::vk::SamplerMipmapMode,

    #[cfg(debug_assertions)]
    debug_name: CString,
}

impl From<Filter> for ash::vk::Filter {
    fn from(value: Filter) -> Self {
        match value {
            Filter::Nearest => ash::vk::Filter::NEAREST,
            Filter::Linear => ash::vk::Filter::LINEAR,
        }
    }
}

impl From<MipmapMode> for ash::vk::SamplerMipmapMode {
    fn from(value: MipmapMode) -> Self {
        match value {
            MipmapMode::Nearest => ash::vk::SamplerMipmapMode::NEAREST,
            MipmapMode::Linear => ash::vk::SamplerMipmapMode::LINEAR,
        }
    }
}

impl From<SamplerAddressMode> for ash::vk::SamplerAddressMode {
    fn from(value: SamplerAddressMode) -> Self {
        match value {
            SamplerAddressMode::Repeat => ash::vk::SamplerAddressMode::REPEAT,
            SamplerAddressMode::MirroredRepeat => ash::vk::SamplerAddressMode::MIRRORED_REPEAT,
            SamplerAddressMode::ClampToEdge => ash::vk::SamplerAddressMode::CLAMP_TO_EDGE,
            SamplerAddressMode::ClampToBorder => ash::vk::SamplerAddressMode::CLAMP_TO_BORDER,
        }
    }
}

impl From<MVSamplerCreateInfo> for CreateInfo {
    fn from(value: MVSamplerCreateInfo) -> Self {
        CreateInfo {
            address_mode: value.address_mode.into(),
            filter_mode: value.filter_mode.into(),
            mipmap_mode: value.mipmap_mode.into(),

            #[cfg(debug_assertions)]
            debug_name: to_ascii_cstring(value.label.unwrap_or_default()),
        }
    }
}

impl VkSampler {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: CreateInfo) -> Self {
        let create_info_vk = ash::vk::SamplerCreateInfo::builder()
            .mag_filter(create_info.filter_mode)
            .min_filter(create_info.filter_mode)
            .mipmap_mode(create_info.mipmap_mode)
            .address_mode_u(create_info.address_mode)
            .address_mode_v(create_info.address_mode)
            .address_mode_w(create_info.address_mode)
            .mip_lod_bias(0.0f32)
            .compare_op(ash::vk::CompareOp::ALWAYS)
            .min_lod(0.0f32)
            .max_lod(ash::vk::LOD_CLAMP_NONE)
            .border_color(ash::vk::BorderColor::FLOAT_OPAQUE_BLACK)
            .max_anisotropy(
                device
                    .get_properties()
                    .properties
                    .limits
                    .max_sampler_anisotropy,
            )
            .anisotropy_enable(false) // TODO
            .unnormalized_coordinates(false)
            .compare_enable(false);

        let handle = unsafe { device.get_device().create_sampler(&create_info_vk, None) }
            .unwrap_or_else(|e| {
                log::error!("Failed to create sampler, error: {e}");
                panic!();
            });

        Self { device, handle }
    }

    pub(crate) fn get_handle(&self) -> ash::vk::Sampler {
        self.handle
    }
}

impl Drop for VkSampler {
    fn drop(&mut self) {
        unsafe { self.device.get_device().destroy_sampler(self.handle, None) };
    }
}
