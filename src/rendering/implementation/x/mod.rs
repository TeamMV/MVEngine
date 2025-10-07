use gpu_alloc::UsageFlags;
use mvutils::version::Version;
use shaderc::{Error, ShaderKind};
use crate::rendering::api::err::RenderingError;
use crate::rendering::api::ShaderType;
use crate::rendering::backend::{Backend, Extent2D};
use crate::rendering::backend::buffer::MemoryProperties;
use crate::rendering::backend::device::{Device, Extensions, MVDeviceCreateInfo};
use crate::rendering::backend::image::{Image, ImageAspect, ImageFormat, ImageTiling, ImageType, ImageUsage, MVImageCreateInfo};
use crate::rendering::backend::shader::Shader;
use crate::window::Window;

pub struct XRendererImpl {
    device: Device
}

impl XRendererImpl {
    /// Create a new instance of our amazing new modern fast and performant XRenderer!
    pub fn new(window: &Window, app_name: &str, version: Version) -> Self {
        let device = Device::new(Backend::Vulkan, MVDeviceCreateInfo {
            app_name: app_name.to_string(),
            app_version: version,
            engine_name: "MVEngine".to_string(),
            engine_version: Version::new(0, 1, 0, 0),
            device_extensions: Extensions::DESCRIPTOR_INDEXING,
        }, window.get_handle());

        Self {
            device,
        }
    }

    /// Compile a new shader using shaderc and return the abstract Shader instance.
    pub fn load_shader(&self, name: &str, ty: ShaderType, source: &str) -> Result<Shader, RenderingError> {
        let kind = match ty {
            ShaderType::Vertex => ShaderKind::Vertex,
            ShaderType::Fragment => ShaderKind::Fragment,
            ShaderType::Compute => ShaderKind::Compute
        };
        let r = Shader::compile(self.device.clone(), source, kind, stropt(name));
        match r {
            Ok(s) => Ok(s),
            Err(e) => Err(RenderingError::ShaderError(e))
        }
    }

    pub fn load_texture(&self, name: &str, data: &[u8], memory_properties: MemoryProperties, usage: ImageUsage, memory_usage_flags: UsageFlags) -> Result<Image, RenderingError> {
        //todo for max: load the image with image crate and extract like dimensions, format and all the shittyty shittaton
        // thats like 2 of the 10 properties we need

        let image = image::load_from_memory(data).map_err(|e| RenderingError::ImageError(e))?.into_rgba8();

        Image::new(self.device.clone(), MVImageCreateInfo {
            size: Extent2D {
                width: image.width(),
                height: image.height(),
            },
            format: ImageFormat::R8G8B8A8,
            usage,
            memory_properties,
            aspect: ImageAspect::COLOR,
            tiling: ImageTiling::Optimal,
            layer_count: 1,
            image_type: ImageType::Image2D,
            cubemap: false,
            memory_usage_flags,
            data: Some(image.into_raw()),
            label: stropt(name),
        })
    }

    pub fn create_texture_manually(&self, create_info: MVImageCreateInfo) -> Result<Image, RenderingError> {
        Image::new(self.device.clone(), create_info)
    }
}

fn stropt(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}