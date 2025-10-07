use crate::rendering::backend::command_buffer::CommandBuffer;
use crate::rendering::backend::device::Device;
use crate::rendering::backend::pipeline::{Pipeline, PipelineType};
use crate::rendering::backend::shader::ShaderStage;
use crate::rendering::backend::vulkan::push_constant::VkPushConstant;

pub enum PushConstant<T: Sized> {
    Vulkan(VkPushConstant<T>),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

pub struct MVPushConstantCreateInfo<T: Sized> {
    pub stage: ShaderStage,
    pub value: T,
}

impl<T: Sized> PushConstant<T> {
    pub fn new(device: Device, create_info: MVPushConstantCreateInfo<T>) -> Self {
        match device {
            Device::Vulkan(device) => {
                PushConstant::Vulkan(VkPushConstant::new(device, create_info.into()))
            }
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub fn push<Type: PipelineType>(&self, cmd: &CommandBuffer, pipeline: Pipeline<Type>) {
        match self {
            PushConstant::Vulkan(push_constant) => {
                push_constant.push(cmd.as_vulkan(), pipeline.as_vulkan())
            }
            #[cfg(target_os = "macos")]
            PushConstant::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            PushConstant::DirectX => unimplemented!(),
        }
    }

    pub fn data(&self) -> &T {
        match self {
            PushConstant::Vulkan(push_constant) => push_constant.data(),
            #[cfg(target_os = "macos")]
            PushConstant::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            PushConstant::DirectX => unimplemented!(),
        }
    }

    pub fn data_mut(&mut self) -> &mut T {
        match self {
            PushConstant::Vulkan(push_constant) => push_constant.data_mut(),
            #[cfg(target_os = "macos")]
            PushConstant::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            PushConstant::DirectX => unimplemented!(),
        }
    }

    pub fn replace(&mut self, new_data: T) {
        match self {
            PushConstant::Vulkan(push_constant) => push_constant.replace(new_data),
            #[cfg(target_os = "macos")]
            PushConstant::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            PushConstant::DirectX => unimplemented!(),
        }
    }
}
