use gl::types::GLuint;
use crate::rendering::batch::RenderBatch;
use crate::rendering::shader::OpenGLShader;
use crate::rendering::{PrimitiveRenderer, Quad, Triangle};
use crate::rendering::camera::OrthographicCamera;
use crate::rendering::post::RenderTarget;
use crate::window::Window;

pub struct RenderController {
    default_shader: GLuint,
    batches: Vec<RenderBatch>,
    batch_index: usize
}

impl RenderController {
    pub fn new(default_shader: GLuint) -> Self {
        unsafe {
            Self {
                default_shader,
                batches: vec![RenderBatch::new(default_shader)],
                batch_index: 0,
            }
        }
    }

    fn setup(&mut self) {
        unsafe {
            self.batches = vec![RenderBatch::new(self.default_shader.clone())];
            self.batch_index = 0;
        }
    }

    pub fn push_triangle(&mut self, triangle: Triangle) {
        unsafe {
            let current = &mut self.batches[self.batch_index];
            if current.can_hold_triangle(&triangle) {
                current.push_triangle(triangle);
            } else {
                self.batches.push(RenderBatch::new(self.default_shader.clone()));
                self.batch_index += 1;
                self.push_triangle(triangle);
            }
        }
    }

    pub fn push_quad(&mut self, quad: Quad) {
        unsafe {
            let current = &mut self.batches[self.batch_index];
            if current.can_hold_quad(&quad) {
                current.push_quad(quad);
            } else {
                self.batches.push(RenderBatch::new(self.default_shader.clone()));
                self.batch_index += 1;
                self.push_quad(quad);
            }
        }
    }

    pub fn draw(&mut self, window: &Window, camera: &OrthographicCamera, renderer: &mut impl PrimitiveRenderer, shader: &mut OpenGLShader) {
        renderer.begin_frame();
        for batch in &mut self.batches {
            if !batch.is_empty() {
                batch.draw(window, camera, renderer, shader);
            }
        }
        self.batch_index = 0;
        renderer.end_frame();
    }

    pub fn draw_to_target(&mut self, window: &Window, camera: &OrthographicCamera, renderer: &mut impl PrimitiveRenderer, shader: &mut OpenGLShader) -> RenderTarget {
        shader.use_program();
        let mut render_target = RenderTarget { texture_1: 0, texture_2: 0, framebuffer: 0, renderbuffer: 0, depth_texture: 0 };
        renderer.begin_frame_to_target(&mut render_target);

        for batch in &mut self.batches {
            if !batch.is_empty() {
                batch.draw_to_target(window, camera, renderer, shader, &mut render_target);
            }
        }
        renderer.end_frame_to_target(&mut render_target);
        self.batch_index = 0;
        render_target
    }
}