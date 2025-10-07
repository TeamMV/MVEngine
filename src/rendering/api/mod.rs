use gpu_alloc::UsageFlags;
use mvutils::version::Version;
use crate::rendering::api::err::RenderingError;
use crate::rendering::backend::buffer::MemoryProperties;
use crate::rendering::backend::image::{Image, ImageUsage, MVImageCreateInfo};
use crate::rendering::backend::shader::Shader;
use crate::rendering::implementation::x::XRendererImpl;
use crate::window::Window;

pub mod err;

#[macro_export]
macro_rules! no_l {
    () => {
        panic!("There is currently no poopy fallback renderer, so if your toaster cannot draw, unlucky")
    };
}

pub enum Renderer {
    L(),
    X(XRendererImpl)
}

impl Renderer {
    pub fn new_x(window: &Window, app_name: &str, version: Version) -> Self {
        Self::X(XRendererImpl::new(window, app_name, version))
    }

    pub fn load_shader(&self, ty: ShaderType, source: &str, name: &str) -> Result<MVShader, RenderingError> {
        match self {
            Renderer::L() => no_l!(),
            Renderer::X(x) => x.load_shader(name, ty, source).map(|s| MVShader::X(s))
        }
    }

    pub fn load_texture(&self, name: &str, source: &[u8], memory_properties: MemoryProperties, usage: ImageUsage, memory_usage_flags: UsageFlags) -> Result<MVTexture, RenderingError> {
        match self {
            Renderer::L() => no_l!(),
            Renderer::X(x) => x.load_texture(name, source, memory_properties, usage, memory_usage_flags).map(|t| MVTexture::X(t))
        }
    }

    pub fn create_texture(&self, create_info: MVImageCreateInfo) -> Result<MVTexture, RenderingError> {
        match self {
            Renderer::L() => no_l!(),
            Renderer::X(x) => x.create_texture_manually(create_info).map(|t| MVTexture::X(t))
        }
    }
}

pub enum MVShader {
    L(),
    X(Shader)
}

pub enum ShaderType {
    Vertex,
    Fragment,
    Compute
}

pub enum MVTexture {
    L(),
    X(Image)
}