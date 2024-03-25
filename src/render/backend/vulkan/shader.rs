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
            stage: value.stage.into(),
            shader_code: value.code,
            #[cfg(debug_assertions)]
            debug_name: to_ascii_cstring(value.label.unwrap_or("".to_string())),
        }
    }
}

impl From<ShaderStage> for ash::vk::ShaderStageFlags {
    fn from(value: ShaderStage) -> Self {
        match value {
            ShaderStage::Vertex => ash::vk::ShaderStageFlags::VERTEX,
            ShaderStage::TesselationControl => ash::vk::ShaderStageFlags::TESSELLATION_CONTROL,
            ShaderStage::TesselationEvaluation => {
                ash::vk::ShaderStageFlags::TESSELLATION_EVALUATION
            }
            ShaderStage::Geometry => ash::vk::ShaderStageFlags::GEOMETRY,
            ShaderStage::Fragment => ash::vk::ShaderStageFlags::FRAGMENT,
            ShaderStage::Compute => ash::vk::ShaderStageFlags::COMPUTE,
            #[cfg(feature = "ray-tracing")]
            ShaderStage::RayGen => ash::vk::ShaderStageFlags::RAYGEN_KHR,
            #[cfg(feature = "ray-tracing")]
            ShaderStage::Miss => ash::vk::ShaderStageFlags::MISS_KHR,
            #[cfg(feature = "ray-tracing")]
            ShaderStage::AnyHit => ash::vk::ShaderStageFlags::ANY_HIT_KHR,
            #[cfg(feature = "ray-tracing")]
            ShaderStage::ClosestHit => ash::vk::ShaderStageFlags::CLOSEST_HIT_KHR,
            #[cfg(feature = "ray-tracing")]
            ShaderStage::Intersection => ash::vk::ShaderStageFlags::INTERSECTION_KHR,
            #[cfg(feature = "ray-tracing")]
            ShaderStage::Callable => ash::vk::ShaderStageFlags::CALLABLE_KHR,
        }
    }
}

pub(crate) struct VkShader {
    stage: ash::vk::ShaderStageFlags,
    handle: ash::vk::ShaderModule,
}

impl VkShader {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: CreateInfo) -> Self {
        let vk_create_info = ash::vk::ShaderModuleCreateInfo::builder()
            .code(&create_info.shader_code)
            .build();

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
            handle: module,
            stage: create_info.stage,
        }
    }

    pub fn create_stage_create_info(&self) -> ash::vk::PipelineShaderStageCreateInfo {
        ash::vk::PipelineShaderStageCreateInfo::builder()
            .stage(self.stage)
            .module(self.handle)
            .name(ENTRY.as_c_str())
            .build()
    }
}
