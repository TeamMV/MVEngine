pub mod adaptive;

use crate::rendering::camera::OrthographicCamera;
use crate::rendering::control::RenderController;
use crate::rendering::shader::default::DefaultOpenGLShader;
use crate::rendering::{OpenGLRenderer, Quad, RenderContext, Triangle};
use crate::rendering::backbuffer::BackBufferTarget;
use crate::ui::geometry::SimpleRect;
use crate::ui::styles::InheritSupplier;
use crate::window::Window;

pub trait WideRenderContext: RenderContext + InheritSupplier {
    fn dpi(&self) -> u32;
}

pub struct UiRenderer {
    last_z: f32,
    renderer: OpenGLRenderer,
    shader: DefaultOpenGLShader,
    controller: RenderController,
    camera: OrthographicCamera,
    dimension: (u32, u32),
    dpi: u32,
}

impl UiRenderer {
    pub fn new(window: &mut Window) -> Self {
        unsafe {
            let mut shader = DefaultOpenGLShader::new();
            shader.make().unwrap();
            shader.bind().unwrap();

            Self {
                last_z: 99.0,
                renderer: OpenGLRenderer::initialize(window),
                controller: RenderController::new(shader.get_program_id()),
                shader,
                camera: OrthographicCamera::new(window.info.width, window.info.height),
                dimension: (window.info.width, window.info().height),
                dpi: window.dpi(),
            }
        }
    }

    pub fn area(&self) -> SimpleRect {
        SimpleRect::new(0, 0, self.dimension.0 as i32, self.dimension.1 as i32)
    }

    pub fn add_triangle(&mut self, triangle: Triangle) {
        self.controller.push_triangle(triangle);
    }

    pub fn add_quad(&mut self, quad: Quad) {
        self.controller.push_quad(quad);
    }

    pub fn draw(&mut self, window: &Window) {
        self.last_z = 99.0;

        self.shader.use_program();
        self.controller
            .draw(window, &self.camera, &mut self.renderer, &mut self.shader, &mut BackBufferTarget::Screen);
    }

    pub fn resize(&mut self, window: &mut Window) {
        unsafe {
            let width = window.info.width;
            let height = window.info.height;
            self.renderer = OpenGLRenderer::initialize(window);
            self.camera = OrthographicCamera::new(width, height);
            self.dimension = (width, height);
        }
    }

    pub fn get_extent(&self) -> (u32, u32) {
        self.dimension
    }

    pub fn controller(&self) -> &RenderController {
        &self.controller
    }

    pub fn controller_mut(&mut self) -> &mut RenderController {
        &mut self.controller
    }
}

impl RenderContext for UiRenderer {
    fn controller(&mut self) -> &mut RenderController {
        &mut self.controller
    }

    fn next_z(&mut self) -> f32 {
        self.controller.next_z()
    }

    fn set_z(&mut self, z: f32) {
        self.controller.set_z(z);
    }
}

impl InheritSupplier for UiRenderer {
    fn x(&self) -> i32 {
        0
    }

    fn y(&self) -> i32 {
        0
    }

    fn width(&self) -> i32 {
        self.dimension.0 as i32
    }

    fn height(&self) -> i32 {
        self.dimension.1 as i32
    }
}

impl WideRenderContext for UiRenderer {
    fn dpi(&self) -> u32 {
        self.dpi
    }
}
