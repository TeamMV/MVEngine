use std::sync::Arc;

use bitflags::bitflags;
use mvcore_proc_macro::graphics_item;
use parking_lot::Mutex;

use crate::render::backend::buffer::Buffer;
use crate::render::backend::command_buffer::CommandBuffer;
use crate::render::backend::device::Device;
use crate::render::backend::image::{Image, ImageLayout};
use crate::render::backend::pipeline::{Pipeline, PipelineType};
use crate::render::backend::sampler::Sampler;
use crate::render::backend::shader::ShaderStage;
use crate::render::backend::vulkan::descriptor_set::VkDescriptorSet;
use crate::render::backend::vulkan::descriptors::descriptor_pool::VkDescriptorPool;
use crate::render::backend::vulkan::descriptors::descriptor_set_layout::VkDescriptorSetLayout;

#[derive(Copy, Clone)]
pub enum DescriptorType {
    CombinedImageSampler,
    StorageImage,
    UniformBuffer,
    StorageBuffer,
    #[cfg(feature = "ray-tracing")]
    AccelerationStructure,
}

pub struct DescriptorSetLayoutBinding {
    pub index: u32,
    pub stages: ShaderStage,
    pub ty: DescriptorType,
    pub count: u32,
}

pub struct MVDescriptorSetLayoutCreateInfo {
    pub bindings: Vec<DescriptorSetLayoutBinding>,

    pub label: Option<String>,
}

#[graphics_item(clone)]
#[derive(Clone)]
pub enum DescriptorSetLayout {
    Vulkan(Arc<VkDescriptorSetLayout>),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl DescriptorSetLayout {
    pub fn new(device: Device, create_info: MVDescriptorSetLayoutCreateInfo) -> Self {
        match device {
            Device::Vulkan(device) => DescriptorSetLayout::Vulkan(
                VkDescriptorSetLayout::new(device, create_info.into()).into(),
            ),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub fn get_binding_count(&self) -> u32 {
        match self {
            DescriptorSetLayout::Vulkan(descriptor_set_layout) => {
                descriptor_set_layout.get_bindings_count()
            }
            #[cfg(target_os = "macos")]
            DescriptorSetLayout::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            DescriptorSetLayout::DirectX => unimplemented!(),
        }
    }
}

pub struct DescriptorPoolSize {
    pub ty: DescriptorType,
    pub count: u32,
}

bitflags! {
    pub struct DescriptorPoolFlags: u8 {
        const FREE_DESCRIPTOR = 1;
        const UPDATE_AFTER_BIND = 1 << 1;
    }
}

pub struct MVDescriptorPoolCreateInfo {
    pub sizes: Vec<DescriptorPoolSize>,
    pub max_sets: u32,
    pub flags: DescriptorPoolFlags,

    pub label: Option<String>,
}

#[graphics_item(clone)]
#[derive(Clone)]
pub enum DescriptorPool {
    Vulkan(Arc<Mutex<VkDescriptorPool>>),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl DescriptorPool {
    pub fn new(device: Device, create_info: MVDescriptorPoolCreateInfo) -> Self {
        match device {
            Device::Vulkan(device) => DescriptorPool::Vulkan(
                Mutex::new(VkDescriptorPool::new(device, create_info.into())).into(),
            ),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }
}

pub struct MVDescriptorSetCreateInfo {
    pub pool: DescriptorPool,
    pub bindings: Vec<DescriptorSetLayoutBinding>,

    pub label: Option<String>,
}

pub struct MVDescriptorSetFromLayoutCreateInfo {
    pub pool: DescriptorPool,
    pub layout: DescriptorSetLayout,

    pub label: Option<String>,
}

#[graphics_item(ref)]
pub enum DescriptorSet {
    Vulkan(VkDescriptorSet),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl DescriptorSet {
    pub fn new(device: Device, create_info: MVDescriptorSetCreateInfo) -> Self {
        match device {
            Device::Vulkan(device) => {
                DescriptorSet::Vulkan(VkDescriptorSet::new(device, create_info.into()))
            }
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub fn from_layout(device: Device, create_info: MVDescriptorSetFromLayoutCreateInfo) -> Self {
        match device {
            Device::Vulkan(device) => {
                DescriptorSet::Vulkan(VkDescriptorSet::from_layout(device, create_info.into()))
            }
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub fn add_buffer(&mut self, binding: u32, buffer: &Buffer, offset: u64, size: u64) {
        match self {
            DescriptorSet::Vulkan(descriptor_set) => descriptor_set.add_buffer(
                binding,
                ash::vk::DescriptorBufferInfo {
                    buffer: buffer.as_vulkan().get_buffer(),
                    offset,
                    range: size,
                },
            ),
            #[cfg(target_os = "macos")]
            DescriptorSet::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            DescriptorSet::DirectX => unimplemented!(),
        }
    }

    pub fn add_image(
        &mut self,
        binding: u32,
        image: &Image,
        sampler: &Sampler,
        layout: ImageLayout,
    ) {
        match self {
            DescriptorSet::Vulkan(descriptor_set) => {
                let image = image.as_vulkan();
                descriptor_set.add_image(
                    binding,
                    ash::vk::DescriptorImageInfo {
                        sampler: sampler.as_vulkan().get_handle(),
                        image_view: image.get_view(0),
                        image_layout: layout.into(),
                    },
                )
            }
            #[cfg(target_os = "macos")]
            DescriptorSet::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            DescriptorSet::DirectX => unimplemented!(),
        }
    }

    pub fn update_buffer(&mut self, binding: u32, buffer: &Buffer, offset: u64, size: u64) {
        match self {
            DescriptorSet::Vulkan(descriptor_set) => descriptor_set.update_buffer(
                binding,
                ash::vk::DescriptorBufferInfo {
                    buffer: buffer.as_vulkan().get_buffer(),
                    offset,
                    range: size,
                },
            ),
            #[cfg(target_os = "macos")]
            DescriptorSet::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            DescriptorSet::DirectX => unimplemented!(),
        }
    }

    pub fn update_image(
        &mut self,
        binding: u32,
        image: &Image,
        sampler: &Sampler,
        layout: ImageLayout,
    ) {
        match self {
            DescriptorSet::Vulkan(descriptor_set) => {
                let image = image.as_vulkan();
                descriptor_set.update_image(
                    binding,
                    0,
                    ash::vk::DescriptorImageInfo {
                        sampler: sampler.as_vulkan().get_handle(),
                        image_view: image.get_view(0),
                        image_layout: layout.into(),
                    },
                )
            }
            #[cfg(target_os = "macos")]
            DescriptorSet::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            DescriptorSet::DirectX => unimplemented!(),
        }
    }

    pub fn update_image_array(
        &mut self,
        binding: u32,
        array_index: u32,
        image: &Image,
        sampler: &Sampler,
        layout: ImageLayout,
    ) {
        match self {
            DescriptorSet::Vulkan(descriptor_set) => {
                let image = image.as_vulkan();
                descriptor_set.update_image(
                    binding,
                    array_index,
                    ash::vk::DescriptorImageInfo {
                        sampler: sampler.as_vulkan().get_handle(),
                        image_view: image.get_view(0),
                        image_layout: layout.into(),
                    },
                )
            }
            #[cfg(target_os = "macos")]
            DescriptorSet::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            DescriptorSet::DirectX => unimplemented!(),
        }
    }

    pub fn build(&mut self) {
        match self {
            DescriptorSet::Vulkan(descriptor_set) => descriptor_set.build(),
            #[cfg(target_os = "macos")]
            DescriptorSet::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            DescriptorSet::DirectX => unimplemented!(),
        }
    }

    pub fn get_layout(&self) -> DescriptorSetLayout {
        match self {
            DescriptorSet::Vulkan(descriptor_set) => {
                DescriptorSetLayout::Vulkan(descriptor_set.get_layout())
            }
            #[cfg(target_os = "macos")]
            DescriptorSet::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            DescriptorSet::DirectX => unimplemented!(),
        }
    }

    pub fn bind<Type: PipelineType + 'static>(
        &mut self,
        command_buffer: &CommandBuffer,
        pipeline: &Pipeline<Type>,
        set_index: u32,
    ) {
        match self {
            DescriptorSet::Vulkan(descriptor_set) => {
                descriptor_set.bind(set_index, pipeline.as_vulkan(), command_buffer.as_vulkan())
            }
            #[cfg(target_os = "macos")]
            DescriptorSet::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            DescriptorSet::DirectX => unimplemented!(),
        }
    }
}
