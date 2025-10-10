use crate::rendering::backend::buffer::MemoryProperties;
use crate::rendering::backend::image::{Image, ImageUsage, MVImageCreateInfo};
use crate::rendering::backend::shader::Shader;
use crate::rendering::implementation::x::XRenderer;
use crate::rendering::implementation::x::core::XRendererCore;
use crate::window::Window;
use bitflags::bitflags;
use gpu_alloc::UsageFlags;
use mvutils::version::Version;

pub mod err;

#[macro_export]
macro_rules! no_l {
    () => {
        panic!(
            "There is currently no poopy fallback renderer, so if your toaster cannot draw, unlucky"
        )
    };
}

bitflags! {
    pub struct RendererCreateInfoFlags: u8 {
        const GREEN_ECO_MODE = 1;
        const VSYNC = 1 << 1;
    }

    pub struct RendererFlavor: u8 {
        const FLAVOR_2D = 1;
        const FLAVOR_3D = 1 << 1;
    }
}

pub struct RendererCreateInfo {
    pub app_name: String,
    pub version: Version,
    pub flags: RendererCreateInfoFlags,
    pub flavor: RendererFlavor,
    pub frames_in_flight: u32,
}

impl Default for RendererCreateInfo {
    fn default() -> Self {
        Self {
            app_name: "Hello Application".to_string(),
            version: Version::new(0, 1, 0, 0),
            flags: RendererCreateInfoFlags::VSYNC,
            flavor: RendererFlavor::FLAVOR_2D,
            frames_in_flight: 2,
        }
    }
}

pub enum Renderer {
    L(),
    X(XRenderer),
}

impl Renderer {
    pub fn new_x(window: &Window, create_info: RendererCreateInfo) -> Option<Self> {
        XRenderer::new(window, create_info).map(|r| Self::X(r))
    }

    pub fn new(window: &Window, create_info: RendererCreateInfo) -> Self {
        match XRenderer::new(window, create_info) {
            //in the future create the opengl renderer here
            // There is currently no poopy fallback renderer, so your toaster cannot draw
            // i love that error so much
            // watch it appear on the error message box
            // on your toaster
            None => no_l!(),
            Some(xr) => Renderer::X(xr),
        }
    }

    pub fn load_shader(
        &self,
        ty: ShaderFlavor,
        source: &str,
        name: &str,
    ) -> Result<MVShader, shaderc::Error> {
        match self {
            Renderer::L() => no_l!(),
            Renderer::X(x) => x.load_shader(name, ty, source).map(|s| MVShader::X(s)),
        }
    }

    pub fn load_texture(
        &self,
        name: &str,
        source: &[u8],
        memory_properties: MemoryProperties,
        usage: ImageUsage,
        memory_usage_flags: UsageFlags,
    ) -> MVTexture {
        match self {
            Renderer::L() => no_l!(),
            Renderer::X(x) => MVTexture::X(x.load_texture(
                name,
                source,
                memory_properties,
                usage,
                memory_usage_flags,
            )),
        }
    }

    pub fn create_texture(&self, create_info: MVImageCreateInfo) -> MVTexture {
        match self {
            Renderer::L() => no_l!(),
            Renderer::X(x) => MVTexture::X(x.create_texture_manually(create_info)),
        }
    }
}

pub enum MVShader {
    L(),
    X(Shader),
}

pub enum ShaderFlavor {
    Vertex,
    Fragment,
    Compute,
}

pub enum MVTexture {
    L(),
    X(Image),
}
