use crate::rendering::backend::device::Device;
use crate::rendering::backend::vulkan::shader::VkShader;
use bitflags::bitflags;
use mvengine_proc_macro::graphics_item;
use shaderc::ShaderKind;
use std::sync::Arc;

pub struct MVShaderCreateInfo {
    pub stage: ShaderStage,
    pub code: Vec<u32>,

    pub label: Option<String>,
}

bitflags! {
     pub struct ShaderStage: u32 {
        const Vertex = 1;
        const TesselationControl = 1 << 1;
        const TesselationEvaluation = 1 << 2;
        const Geometry = 1 << 3;
        const Fragment = 1 << 4;
        const Compute = 1 << 5;
        #[cfg(feature = "ray-tracing")]
        const RayGen = 1 << 8;
        #[cfg(feature = "ray-tracing")]
        const AnyHit = 1 << 9;
        #[cfg(feature = "ray-tracing")]
        const ClosestHit = 1 << 10;
        #[cfg(feature = "ray-tracing")]
        const Miss = 1 << 11;
        #[cfg(feature = "ray-tracing")]
        const Intersection = 1 << 12;
        #[cfg(feature = "ray-tracing")]
        const Callable = 1 << 13;
    }
}

impl From<ShaderKind> for ShaderStage {
    fn from(value: ShaderKind) -> Self {
        match value {
            ShaderKind::Vertex => ShaderStage::Vertex,
            ShaderKind::Fragment => ShaderStage::Fragment,
            ShaderKind::Compute => ShaderStage::Compute,
            ShaderKind::Geometry => ShaderStage::Geometry,
            ShaderKind::TessControl => ShaderStage::TesselationControl,
            ShaderKind::TessEvaluation => ShaderStage::TesselationEvaluation,
            ShaderKind::DefaultVertex => ShaderStage::Vertex,
            ShaderKind::DefaultFragment => ShaderStage::Fragment,
            ShaderKind::DefaultCompute => ShaderStage::Compute,
            ShaderKind::DefaultGeometry => ShaderStage::Geometry,
            ShaderKind::DefaultTessControl => ShaderStage::TesselationControl,
            ShaderKind::DefaultTessEvaluation => ShaderStage::TesselationEvaluation,
            #[cfg(feature = "ray-tracing")]
            ShaderKind::RayGeneration => ShaderStage::RayGen,
            #[cfg(feature = "ray-tracing")]
            ShaderKind::AnyHit => ShaderStage::AnyHit,
            #[cfg(feature = "ray-tracing")]
            ShaderKind::ClosestHit => ShaderStage::ClosestHit,
            #[cfg(feature = "ray-tracing")]
            ShaderKind::Miss => ShaderStage::Miss,
            #[cfg(feature = "ray-tracing")]
            ShaderKind::Intersection => ShaderStage::Intersection,
            #[cfg(feature = "ray-tracing")]
            ShaderKind::Callable => ShaderStage::Callable,
            _ => unimplemented!(),
        }
    }
}

#[graphics_item(clone)]
#[derive(Clone)]
pub enum Shader {
    Vulkan(Arc<VkShader>),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl Shader {
    pub fn new(device: Device, create_info: MVShaderCreateInfo) -> Shader {
        match device {
            Device::Vulkan(device) => {
                Shader::Vulkan(VkShader::new(device, create_info.into()).into())
            }
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub fn compile(
        device: Device,
        data: &str,
        kind: ShaderKind,
        name: Option<String>,
    ) -> Result<Shader, shaderc::Error> {
        match device {
            Device::Vulkan(device) => Ok(Shader::Vulkan(
                VkShader::compile_shader(device, data, kind, name)?.into(),
            )),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }
}
