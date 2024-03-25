use crate::render::backend::device::Device;
use crate::render::backend::vulkan::shader::VkShader;
use mvcore_proc_macro::graphics_item;

pub(crate) struct MVShaderCreateInfo {
    pub(crate) stage: ShaderStage,
    pub(crate) code: Vec<u32>,

    pub(crate) label: Option<String>,
}

pub(crate) enum ShaderStage {
    Vertex,
    TesselationControl,
    TesselationEvaluation,
    Geometry,
    Fragment,
    Compute,
    #[cfg(feature = "ray-tracing")]
    RayGen,
    #[cfg(feature = "ray-tracing")]
    Miss,
    #[cfg(feature = "ray-tracing")]
    AnyHit,
    #[cfg(feature = "ray-tracing")]
    ClosestHit,
    #[cfg(feature = "ray-tracing")]
    Intersection,
    #[cfg(feature = "ray-tracing")]
    Callable,
}

#[graphics_item(ref)]
pub(crate) enum Shader {
    Vulkan(VkShader),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl Shader {
    pub(crate) fn new(device: Device, create_info: MVShaderCreateInfo) -> Shader {
        match device {
            Device::Vulkan(device) => Shader::Vulkan(VkShader::new(device, create_info.into())),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }
}
