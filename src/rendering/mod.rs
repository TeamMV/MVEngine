use crate::math::vec::{Vec2, Vec4};
use crate::rendering::camera::OrthographicCamera;
use crate::rendering::shader::OpenGLShader;
use crate::window::Window;
use gl::types::{GLenum, GLsizei, GLsizeiptr, GLuint, GLuint64};
use std::mem::offset_of;
use std::os::raw::c_void;
use std::ptr::null;
use std::str::FromStr;
use crate::rendering::post::{OpenGLPostProcessRenderer, RenderTarget};

pub mod batch;
pub mod texture;
pub mod shader;
pub mod camera;
pub mod control;
pub mod light;
pub mod post;
pub mod bindless;

#[repr(C)]
#[derive(Clone)]
pub struct Transform {
    pub translation: Vec2,
    pub origin: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            translation: Vec2::default(),
            origin: Vec2::default(),
            scale: Vec2::splat(1.0),
            rotation: 0.0,
        }
    }

    pub fn apply_for_point(&self, point: (i32, i32)) -> (i32, i32) {
        let translated_x = point.0 as f32 - self.origin.x;
        let translated_y = point.1 as f32 - self.origin.y;
        let scaled_x = translated_x * self.scale.x;
        let scaled_y = translated_y * self.scale.y;
        let cos_theta = self.rotation.cos();
        let sin_theta = self.rotation.sin();
        let rotated_x = scaled_x * cos_theta - scaled_y * sin_theta;
        let rotated_y = scaled_x * sin_theta + scaled_y * cos_theta;
        let translated_x = rotated_x + self.origin.x + self.translation.x;
        let translated_y = rotated_y + self.origin.y + self.translation.y;
        (translated_x as i32, translated_y as i32)
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Vertex {
    pub transform: Transform,
    pub pos: (f32, f32, f32),
    pub color: Vec4,
    pub uv: (f32, f32),
    pub texture: f32,
    pub has_texture: f32,
}

impl Vertex {
    pub fn from_inp(value: &InputVertex, tex_idx: f32) -> Self {
        Vertex {
            transform: value.transform.clone(),
            pos: value.pos,
            color: value.color.clone(),
            uv: value.uv,
            texture: tex_idx,
            has_texture: value.has_texture,
        }
    }
}

#[derive(Clone)]
pub struct InputVertex {
    pub transform: Transform,
    pub pos: (f32, f32, f32),
    pub color: Vec4,
    pub uv: (f32, f32),
    pub texture: GLuint,
    pub has_texture: f32,
}

#[derive(Clone)]
pub struct Triangle {
    pub points: [InputVertex; 3],
}

impl Triangle {
    pub fn center(&self) -> (i32, i32) {
        (
            ((self.points[0].pos.0 + self.points[1].pos.0 + self.points[2].pos.0) / 3.0) as i32,
            ((self.points[0].pos.1 + self.points[1].pos.1 + self.points[2].pos.1) / 3.0) as i32,
        )
    }

    pub fn vec2s(&self) -> [Vec2; 3] {
        [
            Vec2::new(self.points[0].pos.0, self.points[0].pos.1),
            Vec2::new(self.points[1].pos.0, self.points[1].pos.1),
            Vec2::new(self.points[2].pos.0, self.points[2].pos.1),
        ]
    }
}

#[derive(Clone)]
pub struct Quad {
    pub points: [InputVertex; 4],
}

pub trait PrimitiveRenderer {
    fn begin_frame(&mut self);
    fn end_frame(&mut self);
    fn begin_frame_to_target(&mut self, post: &mut RenderTarget);
    fn end_frame_to_target(&mut self, post: &mut RenderTarget);
    fn draw_data(&mut self, window: &Window, camera: &OrthographicCamera, vertices: &[u8], indices: &[u32], textures: &[GLuint], vbo: GLuint, ibo: GLuint, amount: u32, amount_textures: usize, shader: &mut OpenGLShader);
    fn draw_data_to_target(&mut self, window: &Window, camera: &OrthographicCamera, vertices: &[u8], indices: &[u32], textures: &[GLuint], vbo: GLuint, ibo: GLuint, amount: u32, amount_textures: usize, shader: &mut OpenGLShader, post: &mut RenderTarget);
}

pub struct OpenGLRenderer;

impl OpenGLRenderer {
    pub unsafe fn initialize(window: &Window) -> Self {
        let handle = window.get_handle();

        handle.make_current().expect("Cannot make OpenGL context current");

        Self {}
    }
}

impl PrimitiveRenderer for OpenGLRenderer {
    fn begin_frame(&mut self) {

    }

    fn end_frame(&mut self) {

    }

    fn begin_frame_to_target(&mut self, post: &mut RenderTarget) {

    }

    fn end_frame_to_target(&mut self, post: &mut RenderTarget) {

    }

    fn draw_data(&mut self, window: &Window, camera: &OrthographicCamera, vertices: &[u8], indices: &[u32], textures: &[GLuint], vbo: GLuint, ibo: GLuint, amount: u32, amount_textures: usize, shader: &mut OpenGLShader) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, vertices.len() as GLsizeiptr, vertices.as_ptr() as *const _, gl::DYNAMIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, indices.len() as GLsizeiptr * 4, indices.as_ptr() as *const _, gl::DYNAMIC_DRAW);

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            shader.uniform_1f("uResX", window.info.width as f32);
            shader.uniform_1f("uResY", window.info.height as f32);
            shader.uniform_matrix_4fv("uProjection", &camera.get_projection());
            shader.uniform_matrix_4fv("uView", &camera.get_view());



            let stride = batch::VERTEX_SIZE_BYTES as GLsizei;

            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, offset_of!(Vertex, transform.translation) as *const c_void);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, offset_of!(Vertex, transform.origin) as *const c_void);
            gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, stride, offset_of!(Vertex, transform.scale) as *const c_void);
            gl::VertexAttribPointer(3, 1, gl::FLOAT, gl::FALSE, stride, offset_of!(Vertex, transform.rotation) as *const c_void);

            gl::VertexAttribPointer(4, 3, gl::FLOAT, gl::FALSE, stride, offset_of!(Vertex, pos) as *const c_void);
            gl::VertexAttribPointer(5, 4, gl::FLOAT, gl::FALSE, stride, offset_of!(Vertex, color) as *const c_void);
            gl::VertexAttribPointer(6, 2, gl::FLOAT, gl::FALSE, stride, offset_of!(Vertex, uv) as *const c_void);
            gl::VertexAttribPointer(7, 1, gl::FLOAT, gl::FALSE, stride, offset_of!(Vertex, texture) as *const c_void);
            gl::VertexAttribPointer(8, 1, gl::FLOAT, gl::FALSE, stride, offset_of!(Vertex, has_texture) as *const c_void);

            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::EnableVertexAttribArray(2);
            gl::EnableVertexAttribArray(3);

            gl::EnableVertexAttribArray(4);
            gl::EnableVertexAttribArray(5);
            gl::EnableVertexAttribArray(6);
            gl::EnableVertexAttribArray(7);
            gl::EnableVertexAttribArray(8);

            gl::DrawElements(gl::TRIANGLES, amount as GLsizei, gl::UNSIGNED_INT, null());

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }

    fn draw_data_to_target(&mut self, window: &Window, camera: &OrthographicCamera, vertices: &[u8], indices: &[u32], textures: &[GLuint], vbo: GLuint, ibo: GLuint, amount: u32, amount_textures: usize, shader: &mut OpenGLShader, post: &mut RenderTarget) {
        todo!()
    }
}

