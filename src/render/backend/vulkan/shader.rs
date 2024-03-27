use crate::render::backend::shader::{MVShaderCreateInfo, ShaderStage};
use crate::render::backend::to_ascii_cstring;
use crate::render::backend::vulkan::device::VkDevice;
use ash::vk::Handle;
use mvutils::lazy;
use std::ffi::CString;
use std::sync::Arc;

lazy! {
    static ENTRY: CString = CString::new("main").unwrap();
}

pub(crate) struct CreateInfo {
    stage: ash::vk::ShaderStageFlags,
    shader_code: Vec<u32>,

    #[cfg(debug_assertions)]
    debug_name: CString,
}

impl From<MVShaderCreateInfo> for CreateInfo {
    fn from(value: MVShaderCreateInfo) -> Self {
        CreateInfo {
            stage: ash::vk::ShaderStageFlags::from_raw(value.stage.bits()),
            shader_code: value.code,
            #[cfg(debug_assertions)]
            debug_name: to_ascii_cstring(value.label.unwrap_or_default()),
        }
    }
}

pub(crate) struct VkShader {
    device: Arc<VkDevice>,

    stage: ash::vk::ShaderStageFlags,
    handle: ash::vk::ShaderModule,
}

impl VkShader {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: CreateInfo) -> Self {
        let vk_create_info =
            ash::vk::ShaderModuleCreateInfo::builder().code(&create_info.shader_code);

        let module = unsafe {
            device
                .get_device()
                .create_shader_module(&vk_create_info, None)
        }
        .unwrap_or_else(|e| {
            log::error!("Failed to create shader module, error: {e}");
            panic!();
        });

        #[cfg(debug_assertions)]
        device.set_object_name(
            &ash::vk::ObjectType::SHADER_MODULE,
            module.as_raw(),
            create_info.debug_name.as_c_str(),
        );

        Self {
            device: device.clone(),
            handle: module,
            stage: create_info.stage,
        }
    }

    pub fn create_stage_create_info(&self) -> ash::vk::PipelineShaderStageCreateInfo {
        ash::vk::PipelineShaderStageCreateInfo {
            stage: self.stage,
            module: self.handle,
            p_name: ENTRY.as_ptr(),
            ..Default::default()
        }
    }
}

impl Drop for VkShader {
    fn drop(&mut self) {
        unsafe {
            self.device
                .get_device()
                .destroy_shader_module(self.handle, None);
        }
    }
}
