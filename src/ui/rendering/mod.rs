pub mod ctx;
pub mod triangle;
pub mod rectangle;
pub mod arc;
pub mod shapes;
pub mod adaptive;

use crate::rendering::control::RenderController;
use crate::rendering::shader::default::DefaultOpenGLShader;
use crate::rendering::{OpenGLRenderer, Quad, Triangle};
use crate::rendering::camera::OrthographicCamera;
use crate::window::Window;

pub struct UiRenderer {
    last_z: f32,
    renderer: OpenGLRenderer,
    shader: DefaultOpenGLShader,
    controller: RenderController,
    camera: OrthographicCamera,
    dimension: (u32, u32)
}

impl UiRenderer {
    pub fn new(window: &mut Window) -> Self {
        unsafe {
            let shader = DefaultOpenGLShader::new();

            Self {
                last_z: 99.0,
                renderer: OpenGLRenderer::initialize(window),
                controller: RenderController::new(shader.get_program_id()),
                shader,
                camera: OrthographicCamera::new(window.info.width, window.info.height),
                dimension: (window.info.width, window.info().height),
            }
        }
    }

    pub(crate) fn gen_z(&mut self) -> f32 {
        let z = self.last_z;
        self.last_z -= 0.005;
        z
    }

    pub fn request_zs(&mut self, amt: usize) -> ZCoords {
        let mut coords = ZCoords::new(amt);
        for _ in 0..amt {
            coords.push_next(self.gen_z());
        }
        coords
    }

    pub fn add_triangle(&mut self, triangle: Triangle) {
        self.controller.push_triangle(triangle);
    }

    pub fn add_quad(&mut self, quad: Quad) {
        self.controller.push_quad(quad);
    }

    pub fn draw(&mut self, window: &Window) {
        self.last_z = 99.0;

        self.controller.draw(window, &self.camera, &mut self.renderer, &mut self.shader);
    }

    pub fn get_extent(&self) -> (u32, u32) {
        self.dimension
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