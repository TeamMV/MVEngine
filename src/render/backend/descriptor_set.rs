use crate::render::backend::buffer::Buffer;
use crate::render::backend::command_buffer::CommandBuffer;
use crate::render::backend::device::Device;
use crate::render::backend::framebuffer::Framebuffer;
use crate::render::backend::image::{Image, ImageLayout};
use crate::render::backend::pipeline::{Pipeline, PipelineType};
use crate::render::backend::sampler::Sampler;
use crate::render::backend::shader::{Shader, ShaderStage};
use crate::render::backend::vulkan::descriptor_set::VkDescriptorSet;
use crate::render::backend::vulkan::descriptors::descriptor_pool::VkDescriptorPool;
use crate::render::backend::vulkan::descriptors::descriptor_set_layout::VkDescriptorSetLayout;
use crate::render::backend::vulkan::shader::VkShader;
use bitflags::bitflags;
use mvcore_proc_macro::graphics_item;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Copy, Clone)]
pub(crate) enum DescriptorType {
    CombinedImageSampler,
    StorageImage,
    UniformBuffer,
    StorageBuffer,
    #[cfg(feature = "ray-tracing")]
    AccelerationStructure,
}

pub(crate) struct DescriptorSetLayoutBinding {
    pub(crate) index: u32,
    pub(crate) stages: ShaderStage,
    pub(crate) ty: DescriptorType,
    pub(crate) count: u32,
}

pub(crate) struct MVDescriptorSetLayoutCreateInfo {
    pub(crate) bindings: Vec<DescriptorSetLayoutBinding>,

    pub(crate) label: Option<String>,
}

#[graphics_item(clone)]
#[derive(Clone)]
pub(crate) enum DescriptorSetLayout {
    Vulkan(Arc<VkDescriptorSetLayout>),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl DescriptorSetLayout {
    pub(crate) fn new(device: Device, create_info: MVDescriptorSetLayoutCreateInfo) -> Self {
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

    pub(crate) fn get_binding_count(&self) -> u32 {
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

pub(crate) struct DescriptorPoolSize {
    pub(crate) ty: DescriptorType,
    pub(crate) count: u32,
}

bitflags! {
    pub(crate) struct DescriptorPoolFlags: u8 {
        const FREE_DESCRIPTOR = 1;
        const UPDATE_AFTER_BIND = 1 << 1;
    }
}

pub(crate) struct MVDescriptorPoolCreateInfo {
    pub(crate) sizes: Vec<DescriptorPoolSize>,
    pub(crate) max_sets: u32,
    pub(crate) flags: DescriptorPoolFlags,

    pub(crate) label: Option<String>,
}

#[graphics_item(clone)]
#[derive(Clone)]
pub(crate) enum DescriptorPool {
    Vulkan(Arc<Mutex<VkDescriptorPool>>),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl DescriptorPool {
    pub(crate) fn new(device: Device, create_info: MVDescriptorPoolCreateInfo) -> Self {
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

pub(crate) struct MVDescriptorSetCreateInfo {
    pub(crate) pool: DescriptorPool,
    pub(crate) bindings: Vec<DescriptorSetLayoutBinding>,

    pub(crate) label: Option<String>,
}

pub(crate) struct MVDescriptorSetFromLayoutCreateInfo {
    pub(crate) pool: DescriptorPool,
    pub(crate) layout: DescriptorSetLayout,

    pub(crate) label: Option<String>,
}

#[graphics_item(ref)]
pub(crate) enum DescriptorSet {
    Vulkan(VkDescriptorSet),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl DescriptorSet {
    pub(crate) fn new(device: Device, create_info: MVDescriptorSetCreateInfo) -> Self {
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

    pub(crate) fn from_layout(
        device: Device,
        create_info: MVDescriptorSetFromLayoutCreateInfo,
    ) -> Self {
        match device {
            Device::Vulkan(device) => DescriptorSet::Vulkan(
                VkDescriptorSet::from_layout(device, create_info.into()).into(),
            ),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn add_buffer(&mut self, binding: u32, buffer: &Buffer, offset: u64, size: u64) {
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

    pub(crate) fn add_image(
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

    pub(crate) fn update_buffer(&mut self, binding: u32, buffer: &Buffer, offset: u64, size: u64) {
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

    pub(crate) fn update_image(
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

    pub(crate) fn build(&mut self) {
        match self {
            DescriptorSet::Vulkan(descriptor_set) => descriptor_set.build(),
            #[cfg(target_os = "macos")]
            DescriptorSet::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            DescriptorSet::DirectX => unimplemented!(),
        }
    }

    pub(crate) fn bind<Type: PipelineType + 'static>(
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
