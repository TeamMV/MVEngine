use crate::rendering::api::{RendererCreateInfo, RendererFlavor, ShaderFlavor};
use crate::rendering::backend::pipeline::Pipeline;
use crate::rendering::backend::shader::Shader;
use crate::rendering::backend::swapchain::SwapchainError;
use crate::rendering::implementation::x::core::XRendererCore;
use crate::rendering::implementation::x::x2d::XRenderer2DAddon;
use crate::rendering::implementation::x::x3d::XRenderer3DAddon;
use crate::window::Window;

pub mod core;
pub mod x2d;
pub mod x3d;

pub struct XRenderer {
    core: XRendererCore,
    // $4.99 in app store
    x2d: Option<XRenderer2DAddon>,
    // $9.99 in app store
    x3d: Option<XRenderer3DAddon>,
}

impl XRenderer {
    pub fn new(window: &Window, create_info: RendererCreateInfo) -> Option<Self> {
        let x2d = if create_info.flavor.contains(RendererFlavor::FLAVOR_2D) {
            Some(XRenderer2DAddon::new())
        } else {
            None
        };

        let x3d = if create_info.flavor.contains(RendererFlavor::FLAVOR_3D) {
            Some(XRenderer3DAddon::new())
        } else {
            None
        };

        Some(Self {
            core: XRendererCore::new(window, create_info)?,
            x2d,
            x3d,
        })
    }

    pub fn draw(&mut self) {
        let image_index = match self.core.begin_draw() {
            Ok(image_index) => image_index,
            Err((i, e)) if matches!(e, SwapchainError::Suboptimal) => i,
            Err((_, e)) => {
                log::info!("Swapchain: {e:?}, waiting for recreation!");
                return;
            }
        };

        if let Some(x3d) = &mut self.x3d {}

        if let Some(x2d) = &mut self.x2d {}

        if let Err(e) = self.core.end_draw() {
            if let SwapchainError::Suboptimal = e {
                //recreate
                self.core.recreate_in_place();
                return;
            }
            log::info!("Swapchain: {e:?}, waiting for recreation!");
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.core.resize(width, height);
    }

    pub fn compile_shader(
        &self,
        name: &str,
        flavor: ShaderFlavor,
        source: &str,
    ) -> Result<Shader, shaderc::Error> {
        self.core.load_shader(name, flavor, source)
    }
}
