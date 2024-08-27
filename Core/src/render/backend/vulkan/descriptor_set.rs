use crate::render::backend::descriptor_set::{
    MVDescriptorSetCreateInfo, MVDescriptorSetFromLayoutCreateInfo,
};
#[cfg(feature = "ray-tracing")]
use crate::render::backend::pipeline::RayTracing;
use crate::render::backend::pipeline::{Compute, Graphics, PipelineType};
use crate::render::backend::vulkan::command_buffer::VkCommandBuffer;
use crate::render::backend::vulkan::descriptors::descriptor_pool::VkDescriptorPool;
use crate::render::backend::vulkan::descriptors::descriptor_set_layout::VkDescriptorSetLayout;
use crate::render::backend::vulkan::descriptors::descriptor_writer::VkDescriptorWriter;
use crate::render::backend::vulkan::descriptors::{descriptor_set_layout, descriptor_writer};
use crate::render::backend::vulkan::device::VkDevice;
use crate::render::backend::vulkan::pipeline::VkPipeline;
use ash::vk::Handle;
use parking_lot::Mutex;
use std::any::TypeId;
use std::sync::Arc;

pub struct VkDescriptorSet {
    device: Arc<VkDevice>,

    handle: ash::vk::DescriptorSet,
    pool: Arc<Mutex<VkDescriptorPool>>,
    pool_index: usize,
    layout: Arc<VkDescriptorSetLayout>,
    bindings_write_info: Vec<Binding>,

    #[cfg(debug_assertions)]
    debug_name: std::ffi::CString,
}

pub(crate) struct CreateInfo {
    pool: Arc<Mutex<VkDescriptorPool>>,
    bindings: Vec<ash::vk::DescriptorSetLayoutBinding>,

    #[cfg(debug_assertions)]
    debug_name: std::ffi::CString,
}

pub(crate) struct FromLayoutCreateInfo {
    pool: Arc<Mutex<VkDescriptorPool>>,
    layout: Arc<VkDescriptorSetLayout>,

    #[cfg(debug_assertions)]
    debug_name: std::ffi::CString,
}

impl From<MVDescriptorSetCreateInfo> for CreateInfo {
    fn from(value: MVDescriptorSetCreateInfo) -> Self {
        CreateInfo {
            pool: value.pool.into_vulkan(),
            bindings: value.bindings.into_iter().map(Into::into).collect(),

            #[cfg(debug_assertions)]
            debug_name: crate::render::backend::to_ascii_cstring(value.label.unwrap_or_default()),
        }
    }
}

impl From<MVDescriptorSetFromLayoutCreateInfo> for FromLayoutCreateInfo {
    fn from(value: MVDescriptorSetFromLayoutCreateInfo) -> Self {
        FromLayoutCreateInfo {
            pool: value.pool.into_vulkan(),
            layout: value.layout.into_vulkan(),

            #[cfg(debug_assertions)]
            debug_name: crate::render::backend::to_ascii_cstring(value.label.unwrap_or_default()),
        }
    }
}

struct Binding {
    descriptor_count: u32,
    binding_data: Vec<BindingData>,
    descriptors_type: ash::vk::DescriptorType,
}

enum BindingData {
    Image(ash::vk::DescriptorImageInfo),
    Buffer(ash::vk::DescriptorBufferInfo),
    #[cfg(feature = "ray-tracing")]
    ASInfo(ash::vk::AccelerationStructureKHR),
}

impl VkDescriptorSet {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: CreateInfo) -> Self {
        let layout_create_info = descriptor_set_layout::CreateInfo {
            bindings: create_info.bindings.clone(),

            #[cfg(debug_assertions)]
            debug_name: create_info.debug_name.clone(),
        };
        let layout = VkDescriptorSetLayout::new(device.clone(), layout_create_info).into();

        Self::from_layout(
            device,
            FromLayoutCreateInfo {
                pool: create_info.pool,
                layout,

                #[cfg(debug_assertions)]
                debug_name: create_info.debug_name,
            },
        )
    }

    pub(crate) fn from_layout(device: Arc<VkDevice>, create_info: FromLayoutCreateInfo) -> Self {
        let mut writes: Vec<Binding> = Vec::new();
        for (writes_index, binding) in create_info.layout.get_bindings().iter().enumerate() {
            match binding.descriptor_type {
                ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER => {
                    writes.push(Binding {
                        descriptor_count: 0,
                        binding_data: Vec::new(),
                        descriptors_type: binding.descriptor_type,
                    });

                    for i in 0..binding.descriptor_count {
                        writes[writes_index].binding_data.push(BindingData::Image(
                            ash::vk::DescriptorImageInfo {
                                sampler: ash::vk::Sampler::null(),
                                image_view: ash::vk::ImageView::null(),
                                image_layout: ash::vk::ImageLayout::UNDEFINED,
                            },
                        ));
                    }
                }
                ash::vk::DescriptorType::STORAGE_IMAGE => {
                    writes.push(Binding {
                        descriptor_count: 0,
                        binding_data: Vec::new(),
                        descriptors_type: binding.descriptor_type,
                    });

                    for i in 0..binding.descriptor_count {
                        writes[writes_index].binding_data.push(BindingData::Image(
                            ash::vk::DescriptorImageInfo {
                                sampler: ash::vk::Sampler::null(),
                                image_view: ash::vk::ImageView::null(),
                                image_layout: ash::vk::ImageLayout::UNDEFINED,
                            },
                        ));
                    }
                }
                ash::vk::DescriptorType::UNIFORM_BUFFER => {
                    writes.push(Binding {
                        descriptor_count: 0,
                        binding_data: Vec::new(),
                        descriptors_type: binding.descriptor_type,
                    });

                    // Push Empty structures
                    for i in 0..binding.descriptor_count {
                        writes[writes_index].binding_data.push(BindingData::Buffer(
                            ash::vk::DescriptorBufferInfo {
                                buffer: ash::vk::Buffer::null(),
                                offset: 0,
                                range: 0,
                            },
                        ));
                    }
                }
                ash::vk::DescriptorType::STORAGE_BUFFER => {
                    writes.push(Binding {
                        descriptor_count: 0,
                        binding_data: Vec::new(),
                        descriptors_type: binding.descriptor_type,
                    });

                    // Push Empty structures
                    for i in 0..binding.descriptor_count {
                        writes[writes_index].binding_data.push(BindingData::Buffer(
                            ash::vk::DescriptorBufferInfo {
                                buffer: ash::vk::Buffer::null(),
                                offset: 0,
                                range: 0,
                            },
                        ));
                    }
                }
                _ => {
                    log::error!(
                        "Descriptor type not supported, type: {}",
                        binding.descriptor_type.as_raw()
                    );
                    panic!();
                }
            }
        }

        Self {
            device,
            handle: ash::vk::DescriptorSet::null(),
            pool: create_info.pool,
            pool_index: 0,
            layout: create_info.layout,
            bindings_write_info: writes,

            #[cfg(debug_assertions)]
            debug_name: create_info.debug_name,
        }
    }

    pub(crate) fn add_buffer(&mut self, binding: u32, buffer_info: ash::vk::DescriptorBufferInfo) {
        // Check if the binding number is valid.
        #[cfg(debug_assertions)]
        if binding > self.layout.get_bindings_count() {
            log::error!(
                "There is no such binding in the layout you provided, last binding is {}",
                self.layout.get_bindings_count()
            );
            panic!();
        }

        // Check if the binding type matches TYPE_UNIFORM_BUFFER or STORAGE_BUFFER.
        #[cfg(debug_assertions)]
        if self.layout.get_binding(binding).descriptor_type.as_raw()
            & (ash::vk::DescriptorType::UNIFORM_BUFFER.as_raw()
                | ash::vk::DescriptorType::STORAGE_BUFFER.as_raw())
            == 0
        {
            log::error!("Binding in the layout has different type, type in the layout: {}. type you want to add: {}", self.layout.get_binding(binding).descriptor_type.as_raw(), self.layout.get_binding(binding).descriptor_type.as_raw());
            panic!();
        }

        // Check if the number of descriptors exceeds the specified count for the binding in the descriptor set layout.
        #[cfg(debug_assertions)]
        if self.layout.get_binding(binding).descriptor_count
            < self.bindings_write_info[binding as usize]
                .binding_data
                .len() as u32
        {
            log::error!(
                "Too many descriptors in the binding, count specified in layout: {}",
                self.layout.get_binding(binding).descriptor_count
            );
            panic!();
        }

        let descriptor_index = self.bindings_write_info[binding as usize].descriptor_count;
        self.bindings_write_info[binding as usize].binding_data[descriptor_index as usize] =
            BindingData::Buffer(buffer_info);

        // increment count for next buffers
        self.bindings_write_info[binding as usize].descriptor_count += 1;
    }

    pub(crate) fn add_image(&mut self, binding: u32, image_info: ash::vk::DescriptorImageInfo) {
        // Check if the binding number is valid.
        #[cfg(debug_assertions)]
        if binding > self.layout.get_bindings_count() {
            log::error!(
                "There is no such binding in the layout you provided, last binding is {}",
                self.layout.get_bindings_count()
            );
            panic!();
        }

        // Check if the binding type matches CombinedImageSampler or STORAGE_IMAGE.
        #[cfg(debug_assertions)]
        if self.layout.get_binding(binding).descriptor_type.as_raw()
            & (ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER.as_raw()
                | ash::vk::DescriptorType::STORAGE_IMAGE.as_raw())
            == 0
        {
            log::error!("Binding in the layout has different type, type in the layout: {}. type you want to add: {}", self.layout.get_binding(binding).descriptor_type.as_raw(), self.layout.get_binding(binding).descriptor_type.as_raw());
            panic!();
        }

        // Check if the number of descriptors exceeds the specified count for the binding in the descriptor set layout.
        #[cfg(debug_assertions)]
        if self.layout.get_binding(binding).descriptor_count
            < self.bindings_write_info[binding as usize]
                .binding_data
                .len() as u32
        {
            log::error!(
                "Too many descriptors in the binding, count specified in layout: {}",
                self.layout.get_binding(binding).descriptor_count
            );
            panic!();
        }

        let descriptor_index = self.bindings_write_info[binding as usize].descriptor_count;
        self.bindings_write_info[binding as usize].binding_data[descriptor_index as usize] =
            BindingData::Image(image_info);

        // increment count for next buffers
        self.bindings_write_info[binding as usize].descriptor_count += 1;
    }

    pub(crate) fn build(&mut self) {
        let writer_create_info = descriptor_writer::CreateInfo {
            layout: self.layout.clone(),
            pool: self.pool.clone(),
        };
        let mut writer = VkDescriptorWriter::new(self.device.clone(), writer_create_info);

        let mut binding_data_buffer = vec![];
        let mut binding_data_image = vec![];

        for (index, binding) in self.bindings_write_info.iter().enumerate() {
            match binding.descriptors_type {
                ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER
                | ash::vk::DescriptorType::STORAGE_IMAGE => {
                    let binding_data = binding
                        .binding_data
                        .iter()
                        .map(|data| {
                            let BindingData::Image(image) = data else {
                                log::error!("Invalid binding data for Image descriptor type");
                                panic!()
                            };
                            *image
                        })
                        .collect::<Vec<_>>();
                    binding_data_image.push(binding_data);
                    writer.write_image(
                        index as u32,
                        0,
                        &binding_data_image[binding_data_image.len() - 1],
                    );
                }
                ash::vk::DescriptorType::STORAGE_BUFFER
                | ash::vk::DescriptorType::UNIFORM_BUFFER => {
                    let binding_data = binding
                        .binding_data
                        .iter()
                        .map(|data| {
                            let BindingData::Buffer(buffer) = data else {
                                log::error!("Invalid binding data for Buffer descriptor type");
                                panic!()
                            };
                            *buffer
                        })
                        .collect::<Vec<_>>();
                    binding_data_buffer.push(binding_data);
                    writer.write_buffer(
                        index as u32,
                        &binding_data_buffer[binding_data_buffer.len() - 1],
                    );
                }
                _ => {}
            }
        }

        writer.build(&mut self.handle, &mut self.pool_index, true);

        #[cfg(debug_assertions)]
        self.device.set_object_name(
            &ash::vk::ObjectType::DESCRIPTOR_SET,
            self.handle.as_raw(),
            self.debug_name.as_c_str(),
        );
    }

    pub(crate) fn update_image(
        &mut self,
        binding: u32,
        array_index: u32,
        image_info: ash::vk::DescriptorImageInfo,
    ) {
        let mut writer = VkDescriptorWriter::new(
            self.device.clone(),
            descriptor_writer::CreateInfo {
                layout: self.layout.clone(),
                pool: self.pool.clone(),
            },
        );

        writer.write_image(binding, array_index, &[image_info]);

        writer.build(&mut self.handle, &mut self.pool_index, false);
    }

    pub(crate) fn update_buffer(
        &mut self,
        binding: u32,
        buffer_info: ash::vk::DescriptorBufferInfo,
    ) {
        let mut writer = VkDescriptorWriter::new(
            self.device.clone(),
            descriptor_writer::CreateInfo {
                layout: self.layout.clone(),
                pool: self.pool.clone(),
            },
        );

        writer.write_buffer(binding, &[buffer_info]);

        writer.build(&mut self.handle, &mut self.pool_index, false);
    }

    pub(crate) fn bind<Type: PipelineType + 'static>(
        &self,
        set_index: u32,
        pipeline: &VkPipeline<Type>,
        cmd: &VkCommandBuffer,
    ) {
        let bind_point = if TypeId::of::<Type>() == TypeId::of::<Graphics>() {
            ash::vk::PipelineBindPoint::GRAPHICS
        } else if TypeId::of::<Type>() == TypeId::of::<Compute>() {
            ash::vk::PipelineBindPoint::COMPUTE
        } else {
            #[cfg(feature = "ray-tracing")]
            if TypeId::of::<Type>() == TypeId::of::<RayTracing>() {
                ash::vk::PipelineBindPoint::RAY_TRACING_KHR
            } else {
                log::error!("Invalid pipeline type");
                panic!()
            }
            #[cfg(not(feature = "ray-tracing"))]
            log::error!("Invalid pipeline type");
            panic!()
        };

        let sets = [self.handle];

        unsafe {
            self.device.get_device().cmd_bind_descriptor_sets(
                cmd.get_handle(),
                bind_point,
                pipeline.get_layout(),
                set_index,
                &sets,
                &[],
            );
        };
    }

    pub(crate) fn get_handle(&self) -> ash::vk::DescriptorSet {
        self.handle
    }

    pub(crate) fn get_layout(&self) -> Arc<VkDescriptorSetLayout> {
        self.layout.clone()
    }
}

impl Drop for VkDescriptorSet {
    fn drop(&mut self) {
        if self.handle.as_raw() != 0 {
            self.pool
                .lock()
                .free_descriptor_sets(self.pool_index, self.handle);
        }
    }
}
