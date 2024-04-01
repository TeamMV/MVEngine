use crate::render::backend::pipeline::PipelineType;
use crate::render::backend::push_constant::MVPushConstantCreateInfo;
use crate::render::backend::vulkan::command_buffer::VkCommandBuffer;
use crate::render::backend::vulkan::device::VkDevice;
use crate::render::backend::vulkan::pipeline::VkPipeline;
use std::sync::Arc;

pub struct VkPushConstant<T: Sized> {
    device: Arc<VkDevice>,
    value: T,

    range: ash::vk::PushConstantRange,
    stage: ash::vk::ShaderStageFlags,
}

pub(crate) struct CreateInfo<T: Sized> {
    stage: ash::vk::ShaderStageFlags,
    value: T,
}

impl<T: Sized> From<MVPushConstantCreateInfo<T>> for CreateInfo<T> {
    fn from(value: MVPushConstantCreateInfo<T>) -> Self {
        CreateInfo {
            stage: ash::vk::ShaderStageFlags::from_raw(value.stage.bits()),
            value: value.value,
        }
    }
}

impl<T: Sized> VkPushConstant<T> {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: CreateInfo<T>) -> Self {
        let range = ash::vk::PushConstantRange {
            stage_flags: create_info.stage,
            offset: 0,
            size: std::mem::size_of::<T>() as u32,
        };

        Self {
            device,
            value: create_info.value,
            range,
            stage: create_info.stage,
        }
    }

    pub(crate) fn push<Type: PipelineType>(
        &self,
        cmd: &VkCommandBuffer,
        pipeline: &VkPipeline<Type>,
    ) {
        let bytes = unsafe {
            std::slice::from_raw_parts(
                &self.value as *const _ as *const u8,
                std::mem::size_of::<T>(),
            )
        };

        unsafe {
            self.device.get_device().cmd_push_constants(
                cmd.get_handle(),
                pipeline.get_layout(),
                self.stage,
                0,
                bytes,
            )
        };
    }

    pub(crate) fn data(&self) -> &T {
        &self.value
    }

    pub(crate) fn data_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub(crate) fn replace(&mut self, data: T) {
        self.value = data;
    }
}
