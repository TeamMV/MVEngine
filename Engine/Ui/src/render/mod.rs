pub mod ctx;
pub mod triangle;
pub mod rectangle;
pub mod arc;
pub mod shapes;

use std::sync::Arc;
use std::vec::IntoIter;
use hashbrown::HashMap;
use itertools::PutBackN;
use mvutils::hashers::U64IdentityHasher;
use mvutils::unsafe_utils::DangerousCell;
use mvutils::version::Version;
use mvcore::render::backend::{Backend, Extent2D};
use mvcore::render::backend::device::{Device, Extensions, MVDeviceCreateInfo};
use mvcore::render::backend::image::{AccessFlags, ImageLayout};
use mvcore::render::backend::swapchain::SwapchainError;
use mvcore::render::renderer::Renderer;
use mvcore::render::texture::Texture;
use mvcore::render::window::Window;
use mvcore::ToAD;
use mve2d::renderer2d::{InputTriangle, Renderer2D, SamplerType};

pub struct UiRenderer {
    device: Device,
    core_renderer: Arc<DangerousCell<Renderer>>,
    renderer2d: Arc<DangerousCell<Renderer2D>>,
    last_z: f32,
    last_texture: u32,
    used_textures: HashMap<u64, u32, U64IdentityHasher>
}

impl UiRenderer {
    pub fn new(window: &mut Window, app_name: String) -> Self {
        let device = Device::new(
            Backend::Vulkan,
            MVDeviceCreateInfo {
                app_name,
                app_version: Version::new(0, 0, 1, 0),
                engine_name: "MVEngine".to_string(),
                engine_version: Version::new(0, 0, 1, 0),
                device_extensions: Extensions::empty(),
            },
            &window.get_handle(),
        );

        let core_renderer = Renderer::new(&window, device.clone()).to_ad();
        let renderer2d = Renderer2D::new(device.clone(), core_renderer.clone(), 0, 0).to_ad();

        Self {
            device,
            core_renderer,
            renderer2d,
            last_z: 99.0,
            last_texture: 0,
            used_textures: HashMap::with_hasher(U64IdentityHasher::default()),
        }
    }

    pub(crate) fn gen_z(&mut self) -> f32 {
        let z = self.last_z;
        self.last_z -= 0.0001;
        z
    }

    pub fn request_zs(&mut self, amt: usize) -> ZCoords {
        let mut coords = ZCoords::new(amt);
        for _ in 0..amt {
            coords.push_next(self.gen_z());
        }
        coords
    }

    pub fn add_triangle(&mut self, triangle: InputTriangle) {
        self.renderer2d.get_mut().add_shape(triangle);
    }

    pub fn set_texture(&mut self, texture: Arc<Texture>, sampler: SamplerType) -> u32 {
        let id = texture.id();
        if self.used_textures.contains_key(&id) {
            return *self.used_textures.get(&id).unwrap();
        }
        let index = self.last_texture;
        self.last_texture += 1;
        self.renderer2d.get_mut().set_texture(index, &texture.image(), sampler);
        self.used_textures.insert(id, index);
        index
    }

    pub fn draw(&mut self) -> Result<(), SwapchainError> {
        self.last_z = 99.0;
        self.last_texture = 0;
        self.used_textures.clear();
        let renderer2d = self.renderer2d.get_mut();
        let image_index = self.core_renderer.get_mut().begin_frame()?;
        let cmd = self.core_renderer.get_mut().get_current_command_buffer();
        let frame_index = self.core_renderer.get().get_current_frame_index();

        renderer2d.draw();

        cmd.blit_image(
            renderer2d.get_geometry_image(frame_index as usize),
            self.core_renderer
                .get_mut()
                .get_swapchain()
                .get_framebuffer(image_index as usize)
                .get_image(0),
        );

        self.core_renderer
            .get_mut()
            .get_swapchain()
            .get_framebuffer(image_index as usize)
            .get_image(0)
            .transition_layout(
                ImageLayout::PresentSrc,
                Some(cmd),
                AccessFlags::empty(),
                AccessFlags::empty(),
            );

        self.core_renderer.get_mut().end_frame()?;
        Ok(())
    }

    pub fn get_device(&self) -> Device {
        self.device.clone()
    }

    pub fn get_extent(&self) -> Extent2D {
        self.renderer2d.get().get_extent().clone()
    }
}

pub struct ZCoords {
    aquired: Vec<f32>,
    amt: usize,
    current: usize,
}

impl ZCoords {
    fn new(amt: usize) -> Self {
        Self {
            aquired: Vec::with_capacity(amt),
            amt,
            current: 0,
        }
    }

    fn push_next(&mut self, next: f32) {
        self.aquired.push(next);
    }

    pub fn next(&mut self) -> f32 {
        if self.current >= self.amt {
            self.current = self.amt - 1;
        }
        let current_idx = self.current;
        self.current += 1;
        self.aquired[current_idx]
    }
}