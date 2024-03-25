use std::ffi::CString;
use std::iter::TrustedRandomAccessNoCoerce;
use std::sync::Arc;
use ash::vk::Handle;
use crate::render::backend::vulkan::descriptors::descriptor_pool::VkDescriptorPool;
use crate::render::backend::vulkan::device::VkDevice;

pub(crate) struct VkDescriptorSetLayout {
    device: Arc<VkDevice>,

    handle: ash::vk::DescriptorSetLayout,
    bindings: Vec<ash::vk::DescriptorSetLayoutBinding>
}

pub(crate) struct CreateInfo {
    pub bindings: Vec<ash::vk::DescriptorSetLayoutBinding>,

    #[cfg(debug_assertions)]
    pub debug_name: CString
}

impl VkDescriptorSetLayout {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: CreateInfo) -> Self {
        let create_info_vk = ash::vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&create_info.bindings)
            .build();

        let handle = unsafe { device.get_device().create_descriptor_set_layout(&create_info_vk, None) }.unwrap_or_else(|e| {
            log::error!("Failed to create descriptor set layout, error: {e}");
            panic!();
        });

        #[cfg(debug_assertions)]
        device.set_object_name(&ash::vk::ObjectType::DESCRIPTOR_SET_LAYOUT, handle.as_raw(), create_info.debug_name.as_c_str());

        Self {
            device,
            handle,
            bindings: create_info.bindings
        }
    }

    pub(crate) fn get_layout(&self) -> ash::vk::DescriptorSetLayout {
        self.handle
    }

    pub(crate) fn get_binding(&self, binding: u32) -> &ash::vk::DescriptorSetLayoutBinding {
        &self.bindings[binding as usize]
    }

    pub(crate) fn get_bindings_count(&self) -> u32 {
        self.bindings.iter().size() as u32
    }


}