use crate::render::backend::descriptor_set::MVDescriptorPoolCreateInfo;
use crate::render::backend::vulkan::descriptors::descriptor_set_layout::VkDescriptorSetLayout;
use crate::render::backend::vulkan::device::VkDevice;
use std::sync::Arc;

pub struct VkDescriptorPool {
    device: Arc<VkDevice>,

    handles: Vec<ash::vk::DescriptorPool>,
    pool_sizes: Vec<ash::vk::DescriptorPoolSize>,
    max_sets: u32,
    pool_flags: ash::vk::DescriptorPoolCreateFlags,

    current_pool_index: u32,

    #[cfg(debug_assertions)]
    debug_name: std::ffi::CString,
}

pub(crate) struct CreateInfo {
    pool_sizes: Vec<ash::vk::DescriptorPoolSize>,
    max_sets: u32,
    pool_flags: ash::vk::DescriptorPoolCreateFlags,

    #[cfg(debug_assertions)]
    debug_name: std::ffi::CString,
}

impl From<MVDescriptorPoolCreateInfo> for CreateInfo {
    fn from(value: MVDescriptorPoolCreateInfo) -> Self {
        CreateInfo {
            pool_sizes: value
                .sizes
                .into_iter()
                .map(|size| ash::vk::DescriptorPoolSize {
                    ty: size.ty.into(),
                    descriptor_count: size.count,
                })
                .collect(),
            max_sets: value.max_sets,
            pool_flags: ash::vk::DescriptorPoolCreateFlags::from_raw(value.flags.bits() as u32),

            #[cfg(debug_assertions)]
            debug_name: crate::render::backend::to_ascii_cstring(value.label.unwrap_or_default()),
        }
    }
}

impl VkDescriptorPool {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: CreateInfo) -> Self {
        let create_info_vk = ash::vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&create_info.pool_sizes)
            .max_sets(create_info.max_sets)
            .flags(create_info.pool_flags);

        let handle = unsafe {
            device
                .get_device()
                .create_descriptor_pool(&create_info_vk, None)
        }
        .unwrap_or_else(|e| {
            log::error!("Failed to uix descriptor pool, error: {e}");
            panic!();
        });

        #[cfg(debug_assertions)]
        device.set_object_name(
            &ash::vk::ObjectType::DESCRIPTOR_POOL,
            ash::vk::Handle::as_raw(handle),
            create_info.debug_name.as_c_str(),
        );

        Self {
            device,
            pool_sizes: create_info.pool_sizes,
            pool_flags: create_info.pool_flags,
            handles: vec![handle],
            max_sets: create_info.max_sets,
            current_pool_index: 0,

            #[cfg(debug_assertions)]
            debug_name: create_info.debug_name,
        }
    }

    pub(crate) fn allocate_descriptor_set(
        &mut self,
        layout: &VkDescriptorSetLayout,
    ) -> (usize, ash::vk::DescriptorSet) {
        let layouts = [layout.get_layout()];
        let alloc_info = ash::vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.handles[self.current_pool_index as usize])
            .set_layouts(&layouts);

        let descriptor = unsafe {
            self.device
                .get_device()
                .allocate_descriptor_sets(&alloc_info)
        }
        .unwrap_or_else(|e| {
            // It is possible for allocation to fail when pool is full, so we try to recreate pool and allocate again

            self.create_new_pool();

            let layouts = [layout.get_layout()];
            let alloc_info = ash::vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(self.handles[self.current_pool_index as usize])
                .set_layouts(&layouts);

            let descriptor = unsafe {
                self.device
                    .get_device()
                    .allocate_descriptor_sets(&alloc_info)
            }
            .unwrap_or_else(|e| {
                // If it fails again we panic
                log::error!("Failed to allocate descriptor set, error: {e}");
                panic!();
            });

            descriptor
        })[0];

        (self.current_pool_index as usize, descriptor)
    }

    pub(crate) fn free_descriptor_sets(&mut self, pool_index: usize, set: ash::vk::DescriptorSet) {
        let sets = [set];
        unsafe {
            self.device
                .get_device()
                .free_descriptor_sets(self.handles[pool_index], &sets)
                .unwrap_or_else(|e| {
                    log::error!("Failed to free descriptor set, error: {e}");
                    panic!();
                });
        }
    }

    fn create_new_pool(&mut self) {
        let create_info_vk = ash::vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&self.pool_sizes)
            .max_sets(self.max_sets)
            .flags(self.pool_flags);

        let handle = unsafe {
            self.device
                .get_device()
                .create_descriptor_pool(&create_info_vk, None)
        }
        .unwrap_or_else(|e| {
            log::error!("Failed to uix descriptor pool, error: {e}");
            panic!();
        });

        #[cfg(debug_assertions)]
        self.device.set_object_name(
            &ash::vk::ObjectType::DESCRIPTOR_POOL,
            ash::vk::Handle::as_raw(handle),
            self.debug_name.as_c_str(),
        );

        self.handles.push(handle);
        self.current_pool_index += 1;
    }
}

impl Drop for VkDescriptorPool {
    fn drop(&mut self) {
        unsafe {
            for handle in &self.handles {
                self.device
                    .get_device()
                    .destroy_descriptor_pool(*handle, None);
            }
        }
    }
}
