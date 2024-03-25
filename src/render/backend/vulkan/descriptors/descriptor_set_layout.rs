use crate::render::backend::vulkan::descriptors::descriptor_pool::VkDescriptorPool;
use crate::render::backend::vulkan::device::VkDevice;
use ash::vk::Handle;
use std::ffi::CString;
use std::sync::Arc;
use crate::render::backend::descriptor_set::{DescriptorSetLayoutBinding, DescriptorType, MVDescriptorSetLayoutCreateInfo};
use crate::render::backend::to_ascii_cstring;

pub(crate) struct VkDescriptorSetLayout {
    device: Arc<VkDevice>,

    handle: ash::vk::DescriptorSetLayout,
    bindings: Vec<ash::vk::DescriptorSetLayoutBinding>,
}

pub(crate) struct CreateInfo {
    pub bindings: Vec<ash::vk::DescriptorSetLayoutBinding>,

    #[cfg(debug_assertions)]
    pub debug_name: CString,
}

impl From<MVDescriptorSetLayoutCreateInfo> for CreateInfo {
    fn from(value: MVDescriptorSetLayoutCreateInfo) -> Self {
        CreateInfo {
            bindings: value.bindings.into_iter().map(Into::into).collect(),

            #[cfg(debug_assertions)]
            debug_name: to_ascii_cstring(value.label.unwrap_or_default()),
        }
    }
}

impl From<DescriptorSetLayoutBinding> for ash::vk::DescriptorSetLayoutBinding {
    fn from(value: DescriptorSetLayoutBinding) -> Self {
        ash::vk::DescriptorSetLayoutBinding {
            binding: value.index,
            descriptor_type: value.ty.into(),
            descriptor_count: value.count,
            stage_flags: ash::vk::ShaderStageFlags::from_raw(value.stages.bits()),
            p_immutable_samplers: std::ptr::null(),
        }
    }
}

impl From<DescriptorType> for ash::vk::DescriptorType {
    fn from(value: DescriptorType) -> Self {
        match value {
            DescriptorType::CombinedImageSampler => ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            DescriptorType::StorageImage => ash::vk::DescriptorType::STORAGE_IMAGE,
            DescriptorType::UniformBuffer => ash::vk::DescriptorType::UNIFORM_BUFFER,
            DescriptorType::StorageBuffer => ash::vk::DescriptorType::STORAGE_BUFFER,
            #[cfg(feature = "ray-tracing")]
            DescriptorType::AccelerationStructure => ash::vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
        }
    }
}

impl VkDescriptorSetLayout {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: CreateInfo) -> Self {
        let create_info_vk = ash::vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&create_info.bindings);

        let handle = unsafe {
            device
                .get_device()
                .create_descriptor_set_layout(&create_info_vk, None)
        }
        .unwrap_or_else(|e| {
            log::error!("Failed to create descriptor set layout, error: {e}");
            panic!();
        });

        #[cfg(debug_assertions)]
        device.set_object_name(
            &ash::vk::ObjectType::DESCRIPTOR_SET_LAYOUT,
            handle.as_raw(),
            create_info.debug_name.as_c_str(),
        );

        Self {
            device,
            handle,
            bindings: create_info.bindings,
        }
    }

    pub(crate) fn get_layout(&self) -> ash::vk::DescriptorSetLayout {
        self.handle
    }

    pub(crate) fn get_binding(&self, binding: u32) -> &ash::vk::DescriptorSetLayoutBinding {
        &self.bindings[binding as usize]
    }

    pub(crate) fn get_bindings(&self) -> &[ash::vk::DescriptorSetLayoutBinding] {
        &self.bindings
    }

    pub(crate) fn get_bindings_count(&self) -> u32 {
        self.bindings.len() as u32
    }
}

impl Drop for VkDescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self.device.get_device().destroy_descriptor_set_layout(self.handle, None);
        }
    }
}

unsafe impl Send for VkDescriptorSetLayout {}
unsafe impl Sync for VkDescriptorSetLayout {}