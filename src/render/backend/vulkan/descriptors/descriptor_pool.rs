use std::any::Any;
use std::ffi::CString;
use std::sync::Arc;
use ash::vk::Handle;
use crate::render::backend::vulkan::descriptors::descriptor_set_layout::VkDescriptorSetLayout;
use crate::render::backend::vulkan::device::VkDevice;

pub(crate) struct VkDescriptorPool {
    device: Arc<VkDevice>,

    handles: Vec<ash::vk::DescriptorPool>,
    pool_sizes: Vec<ash::vk::DescriptorPoolSize>,
    max_sets: u32,
    pool_flags: ash::vk::DescriptorPoolCreateFlags,

    current_pool_index: u32,

    #[cfg(debug_assertions)]
    debug_name: CString
}

pub(crate) struct CreateInfo {
    pool_sizes: Vec<ash::vk::DescriptorPoolSize>,
    max_sets: u32,
    pool_flags: ash::vk::DescriptorPoolCreateFlags,

    debug_name: CString
}

impl VkDescriptorPool {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: CreateInfo) -> Self {

        let create_info_vk = ash::vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&create_info.pool_sizes)
            .max_sets(create_info.max_sets)
            .flags(create_info.pool_flags)
            .build();

        let handle = unsafe { device.get_device().create_descriptor_pool(&create_info_vk, None) }.unwrap_or_else(|e| {
            log::error!("Failed to create descriptor pool, error: {e}");
            panic!();
        });

        #[cfg(debug_assertions)]
        device.set_object_name(&ash::vk::ObjectType::DESCRIPTOR_POOL, handle.as_raw(), create_info.debug_name.as_c_str());

        Self {
            device,
            pool_sizes: create_info.pool_sizes,
            pool_flags: create_info.pool_flags,
            handles: vec![handle],
            max_sets: create_info.max_sets,
            current_pool_index: 0,
            debug_name: create_info.debug_name
        }
    }

    // This function will only be used in descriptor_writer, so we don't really need wrapper for it
    pub(crate) fn allocate_descriptor_set(&mut self, layout: &VkDescriptorSetLayout) -> ash::vk::DescriptorSet {
        let alloc_info = ash::vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.handles[self.current_pool_index as usize])
            .set_layouts(&[layout.get_layout()])
            .build();

        let descriptor = unsafe { self.device.get_device().allocate_descriptor_sets(&alloc_info) }.unwrap_or_else(|e| {
            // It is possible for allocation to fail when pool is full, so we try to recreate pool and allocate again

            self.create_new_pool();

            let alloc_info = ash::vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(self.handles[self.current_pool_index as usize])
                .set_layouts(&[layout.get_layout()])
                .build();

            let descriptor = unsafe { self.device.get_device().allocate_descriptor_sets(&alloc_info) }.unwrap_or_else(|e| {
                // If it fails again we panic
                log::error!("Failed to allocate descriptor set, error: {e}");
                panic!();
            });

            descriptor
        })[0];

        descriptor
    }

    fn create_new_pool(&mut self) {
        let create_info_vk = ash::vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&self.pool_sizes)
            .max_sets(self.max_sets)
            .flags(self.pool_flags)
            .build();

        let handle = unsafe { self.device.get_device().create_descriptor_pool(&create_info_vk, None) }.unwrap_or_else(|e| {
            log::error!("Failed to create descriptor pool, error: {e}");
            panic!();
        });

        #[cfg(debug_assertions)]
        self.device.set_object_name(&ash::vk::ObjectType::DESCRIPTOR_POOL, handle.as_raw(), self.debug_name.as_c_str());

        self.handles.push(handle);
        self.current_pool_index += 1;
    }
}
