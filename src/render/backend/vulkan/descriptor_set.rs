use std::ffi::CString;
use std::iter::TrustedRandomAccessNoCoerce;
use std::sync::Arc;
use ash::vk::{Buffer, DescriptorType};
use log::log;
use mvsync::CommandBuffer;
use crate::render::backend::vulkan::command_buffer::VkCommandBuffer;
use crate::render::backend::vulkan::descriptors::descriptor_pool::VkDescriptorPool;
use crate::render::backend::vulkan::descriptors::descriptor_set_layout::VkDescriptorSetLayout;
use crate::render::backend::vulkan::descriptors::{descriptor_set_layout, descriptor_writer};
use crate::render::backend::vulkan::descriptors::descriptor_writer::VkDescriptorWriter;
use crate::render::backend::vulkan::device::VkDevice;
use crate::render::backend::vulkan::pipeline::VkPipeline;

pub(crate) struct VkDescriptorSet {
    device: Arc<VkDevice>,

    handle: ash::vk::DescriptorSet,
    pool: Arc<VkDescriptorPool>,
    layout: Arc<VkDescriptorSetLayout>,
    bindings_write_info: Vec<Binding>
}

pub(crate) struct CreateInfo {
    pool: Arc<VkDescriptorPool>,
    bindings: Vec<ash::vk::DescriptorSetLayoutBinding>,

    debug_name: CString
}

struct Binding {
    descriptor_count: u32,
    binding_data: Vec<BindingData>,
    descriptors_type: ash::vk::DescriptorType
}

enum BindingData {
    Image(ash::vk::DescriptorImageInfo),
    Buffer(ash::vk::DescriptorBufferInfo),
    ASInfo(ash::vk::AccelerationStructureKHR)
}

impl VkDescriptorSet {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: CreateInfo) -> Self {

        // Create Layout
        let layout_create_info = descriptor_set_layout::CreateInfo {
            bindings: create_info.bindings,
            debug_name: create_info.debug_name.clone()
        };
        let layout = VkDescriptorSetLayout::new(device.clone(), layout_create_info);

        // Fill write_info with empty stuff
        let mut writes: Vec<Binding> = Vec::new();
        let mut writes_index = 0;
        for binding in create_info.bindings {
            match binding.descriptor_type
            {
                DescriptorType(ash::vk::DescriptorType::as_raw(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)) => {
                    writes.push(Binding{
                        descriptor_count: 0,
                        binding_data: Vec::new(),
                        descriptors_type: binding.descriptor_type
                    });

                    // Push Empty structures
                    for i in 0..binding.descriptor_count {
                        writes[writes_index as u32].binding_data.push(
                            BindingData::Image(
                                ash::vk::DescriptorImageInfo::builder()
                                .image_view(ash::vk::ImageView::null())
                                .image_layout(ash::vk::ImageLayout::UNDEFINED)
                                .sampler(ash::vk::Sampler::null())
                                .build()));
                    }
                }
                DescriptorType(ash::vk::DescriptorType::as_raw(ash::vk::DescriptorType::STORAGE_IMAGE)) => {
                    writes.push(Binding{
                        descriptor_count: 0,
                        binding_data: Vec::new(),
                        descriptors_type: binding.descriptor_type
                    });

                    // Push Empty structures
                    for i in 0..binding.descriptor_count {
                        writes[writes_index as u32].binding_data.push(
                            BindingData::Image(
                                ash::vk::DescriptorImageInfo::builder()
                                    .image_view(ash::vk::ImageView::null())
                                    .image_layout(ash::vk::ImageLayout::UNDEFINED)
                                    .sampler(ash::vk::Sampler::null())
                                    .build()));
                    }
                }
                DescriptorType(ash::vk::DescriptorType::as_raw(ash::vk::DescriptorType::UNIFORM_BUFFER)) => {
                    writes.push(Binding{
                        descriptor_count: 0,
                        binding_data: Vec::new(),
                        descriptors_type: binding.descriptor_type
                    });

                    // Push Empty structures
                    for i in 0..binding.descriptor_count {
                        writes[writes_index as u32].binding_data.push(
                            BindingData::Buffer(
                                ash::vk::DescriptorBufferInfo::builder()
                                    .range(0)
                                    .offset(0)
                                    .buffer(ash::vk::Buffer::null())
                                    .build()));
                    }
                }
                DescriptorType(ash::vk::DescriptorType::as_raw(ash::vk::DescriptorType::STORAGE_BUFFER)) => {
                    writes.push(Binding{
                        descriptor_count: 0,
                        binding_data: Vec::new(),
                        descriptors_type: binding.descriptor_type
                    });

                    // Push Empty structures
                    for i in 0..binding.descriptor_count {
                        writes[writes_index as u32].binding_data.push(
                            BindingData::Buffer(
                                ash::vk::DescriptorBufferInfo::builder()
                                    .range(0)
                                    .offset(0)
                                    .buffer(ash::vk::Buffer::null())
                                    .build()));
                    }
                }
                DescriptorType(_) => {
                    log::error!("Descriptor type not supported, type: {}", binding.descriptor_type.as_raw());
                    panic!();
                }
            }

            writes_index += 1;
        }

        Self {
            device,
            handle: ash::vk::DescriptorSet::null(),
            pool: create_info.pool,
            layout: Arc::new(layout),
            bindings_write_info: writes,
        }
    }

    pub(crate) fn add_buffer(&mut self, binding: u32, buffer_info: ash::vk::DescriptorBufferInfo) {

        // Check if the binding number is valid.
        #[cfg(debug_assertions)]
        if binding > self.layout.get_bindings_count() {
            log::error!("There is no such binding in the layout you provided, last binding is {}", self.layout.get_bindings_count());
            panic!();
        }

        // Check if the binding type matches TYPE_UNIFORM_BUFFER or STORAGE_BUFFER.
        #[cfg(debug_assertions)]
        if self.layout.get_binding(binding).descriptor_type & (ash::vk::DescriptorType::UNIFORM_BUFFER ||ash::vk::DescriptorType::STORAGE_BUFFER) == 0 {
            log::error!("Binding in the layout has different type, type in the layout: {}. type you want to add: {}", self.layout.get_binding(binding).descriptor_type.as_raw(), self.layout.get_binding(binding).descriptor_type.as_raw());
            panic!();
        }

        // Check if the number of descriptors exceeds the specified count for the binding in the descriptor set layout.
        #[cfg(debug_assertions)]
        if self.layout.get_binding(binding).descriptor_count < self.bindings_write_info[binding as usize].binding_data.iter().size() as u32 {
            log::error!("Too many descriptors in the binding, count specified in layout: {}", self.layout.get_binding(binding).descriptor_count);
            panic!();
        }

        let descriptor_index = self.bindings_write_info[binding as usize].descriptor_count;
        self.bindings_write_info[binding as usize].binding_data[descriptor_index as usize] = BindingData::Buffer(buffer_info);

        // increment count for next buffers
        self.bindings_write_info[binding as usize].descriptor_count += 1;
    }

    pub(crate) fn add_image(&self, binding: u32, image_info: ash::vk::DescriptorImageInfo) {
        // Check if the binding number is valid.
        #[cfg(debug_assertions)]
        if binding > self.layout.get_bindings_count() {
            log::error!("There is no such binding in the layout you provided, last binding is {}", self.layout.get_bindings_count());
            panic!();
        }

        // Check if the binding type matches COMBINED_IMAGE_SAMPLER or STORAGE_IMAGE.
        #[cfg(debug_assertions)]
        if self.layout.get_binding(binding).descriptor_type & (ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER ||ash::vk::DescriptorType::STORAGE_IMAGE) == 0 {
            log::error!("Binding in the layout has different type, type in the layout: {}. type you want to add: {}", self.layout.get_binding(binding).descriptor_type.as_raw(), self.layout.get_binding(binding).descriptor_type.as_raw());
            panic!();
        }

        // Check if the number of descriptors exceeds the specified count for the binding in the descriptor set layout.
        #[cfg(debug_assertions)]
        if self.layout.get_binding(binding).descriptor_count < self.bindings_write_info[binding as usize].binding_data.iter().size() as u32 {
            log::error!("Too many descriptors in the binding, count specified in layout: {}", self.layout.get_binding(binding).descriptor_count);
            panic!();
        }

        let descriptor_index = self.bindings_write_info[binding as usize].descriptor_count;
        self.bindings_write_info[binding as usize].binding_data[descriptor_index as usize] = BindingData::Image(image_info);

        // increment count for next buffers
        self.bindings_write_info[binding as usize].descriptor_count += 1;
    }

    pub(crate) fn build(&mut self) {
        let writer_create_info = descriptor_writer::CreateInfo {
            layout: self.layout.clone(),
            pool: self.pool.clone()
        };
        let mut writer = VkDescriptorWriter::new(self.device.clone(), writer_create_info);

        let mut index = 0;
        for binding in self.bindings_write_info {
            match binding.descriptors_type {
                DescriptorType(ash::vk::DescriptorType::as_raw(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)) => {
                    writer.write_image(index, &binding.binding_data);
                }
                DescriptorType(ash::vk::DescriptorType::as_raw(ash::vk::DescriptorType::STORAGE_IMAGE)) => {
                    writer.write_image(index, &binding.binding_data);
                }
                DescriptorType(ash::vk::DescriptorType::as_raw(ash::vk::DescriptorType::STORAGE_IMAGE)) => {
                    writer.write_buffer(index, &binding.binding_data);
                }
                DescriptorType(ash::vk::DescriptorType::as_raw(ash::vk::DescriptorType::STORAGE_IMAGE)) => {
                    writer.write_buffer(index, &binding.binding_data);
                }
            }
            index += 1;
        }

        writer.build(&mut self.handle, true);
    }

    pub(crate) fn bind(&self, set_index: u32, pipeline: VkPipeline, cmd: VkCommandBuffer) {
        let bind_point = todo!();

        unsafe { self.device.get_device().cmd_bind_descriptor_sets(
            cmd.get_handle(),
            bind_point,
            pipeline.get_layout(),
            set_index,
            &[self.handle],
            &[]);
        };
    }
}