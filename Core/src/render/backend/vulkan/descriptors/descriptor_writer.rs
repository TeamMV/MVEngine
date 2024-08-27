use crate::render::backend::vulkan::descriptors::descriptor_pool::VkDescriptorPool;
use crate::render::backend::vulkan::descriptors::descriptor_set_layout::VkDescriptorSetLayout;
use crate::render::backend::vulkan::device::VkDevice;
use parking_lot::Mutex;
use std::sync::Arc;

pub(crate) struct VkDescriptorWriter {
    device: Arc<VkDevice>,

    layout: Arc<VkDescriptorSetLayout>,
    pool: Arc<Mutex<VkDescriptorPool>>,
    writes: Vec<ash::vk::WriteDescriptorSet>,
}

pub(crate) struct CreateInfo {
    pub layout: Arc<VkDescriptorSetLayout>,
    pub pool: Arc<Mutex<VkDescriptorPool>>,
}

impl VkDescriptorWriter {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: CreateInfo) -> Self {
        Self {
            device,
            layout: create_info.layout,
            pool: create_info.pool,
            writes: Vec::new(),
        }
    }

    pub(crate) fn write_buffer(
        &mut self,
        binding: u32,
        buffer_info: &[ash::vk::DescriptorBufferInfo],
    ) {
        let binding_description = self.layout.get_binding(binding);

        let write = ash::vk::WriteDescriptorSet {
            dst_binding: binding,
            descriptor_type: binding_description.descriptor_type,
            descriptor_count: buffer_info.len() as u32,
            p_buffer_info: buffer_info.as_ptr(),
            ..Default::default()
        };

        self.writes.push(write);
    }

    pub(crate) fn write_image(
        &mut self,
        binding: u32,
        array_index: u32,
        image_info: &[ash::vk::DescriptorImageInfo],
    ) {
        let binding_description = self.layout.get_binding(binding);

        let mut write = ash::vk::WriteDescriptorSet::builder()
            .descriptor_type(binding_description.descriptor_type)
            .dst_binding(binding)
            .image_info(image_info);

        // Not sure why but this field isn't present in the builder
        write.descriptor_count = image_info.len() as u32;
        write.dst_array_element = array_index;

        self.writes.push(*write);
    }

    pub(crate) fn build(
        &mut self,
        set: &mut ash::vk::DescriptorSet,
        pool_index: &mut usize,
        allocate_set: bool,
    ) {
        if allocate_set {
            let (idx, desc_set) = self.pool.lock().allocate_descriptor_set(&self.layout);
            *set = desc_set;
            *pool_index = idx;
        }

        for write in &mut self.writes {
            write.dst_set = *set;
        }

        unsafe {
            self.device
                .get_device()
                .update_descriptor_sets(&self.writes, &Vec::new())
        };

        self.writes.clear();
    }
}
