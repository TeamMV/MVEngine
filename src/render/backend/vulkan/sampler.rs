use std::ffi::CString;
use std::sync::Arc;
use crate::render::backend::vulkan::device::VkDevice;

pub(crate) struct VkSampler {
    device: Arc<VkDevice>,
    handle: ash::vk::Sampler
}

pub(crate) struct CreateInfo {
    address_mode: ash::vk::SamplerAddressMode,
    filter_mode: ash::vk::Filter,
    mipmap_mode: ash::vk::SamplerMipmapMode,

    #[cfg(debug_assertions)]
    debug_name: CString
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
            .max_anisotropy(device.get_properties().properties.limits.max_sampler_anisotropy)
            .anisotropy_enable(false) // TODO
            .unnormalized_coordinates(false)
            .compare_enable(false);

        let handle = unsafe { device.get_device().create_sampler(&create_info_vk, None) }.unwrap_or_else(|e| {
            log::error!("Failed to create sampler, error: {e}");
            panic!();
        });

        Self {
            device,
            handle,
        }
    }
}