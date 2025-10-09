use crate::rendering::backend::shader::{MVShaderCreateInfo, ShaderStage};
use crate::rendering::backend::vulkan::device::VkDevice;
use mvutils::lazy;
use std::ffi::CString;
use std::sync::Arc;
use shaderc::{OptimizationLevel, ShaderKind, TargetEnv};

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
            debug_name: crate::rendering::backend::to_ascii_cstring(value.label.unwrap_or_default()),
        }
    }
}

pub struct VkShader {
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
            panic!("Critical Vulkan driver ERROR")
        });

        #[cfg(debug_assertions)]
        device.set_object_name(
            &ash::vk::ObjectType::SHADER_MODULE,
            ash::vk::Handle::as_raw(module),
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

    pub fn compile_shader(device: Arc<VkDevice>, data: &str, kind: ShaderKind, name: Option<String>) -> Result<Self, shaderc::Error> {
        let compiler = shaderc::Compiler::new().unwrap();
        let mut options = shaderc::CompileOptions::new().unwrap();
        options.set_optimization_level(OptimizationLevel::Performance);

        options.set_target_env(TargetEnv::Vulkan, ash::vk::API_VERSION_1_2);
        let code = compiler
            .compile_into_spirv(
                data,
                kind,
                name.as_ref().unwrap_or(&"".to_string()),
                "main",
                Some(&options),
            )?
            .as_binary()
            .to_vec();

        Ok(Self::new(device, CreateInfo {
            stage: ash::vk::ShaderStageFlags::from_raw(ShaderStage::from(kind).bits()),
            shader_code: code,
            debug_name: crate::rendering::backend::to_ascii_cstring(name.unwrap_or_default()),
        }))
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
