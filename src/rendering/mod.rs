use crate::color::RgbColor;
use crate::math::vec::{Vec2, Vec4};
use crate::rendering::camera::OrthographicCamera;
use crate::rendering::control::RenderController;
use crate::rendering::light::LightOpenGLRenderer;
use crate::rendering::post::{OpenGLPostProcessRenderer, RenderTarget};
use crate::rendering::shader::OpenGLShader;
use crate::window::Window;
use gl::types::{GLenum, GLint, GLsizei, GLsizeiptr, GLuint, GLuint64};
use std::mem::offset_of;
use std::os::raw::c_void;
use std::ptr::null;
use std::str::FromStr;

pub mod batch;
pub mod bindless;
pub mod camera;
pub mod control;
pub mod light;
pub mod post;
pub mod shader;
pub mod text;
pub mod texture;

pub trait RenderContext {
    fn controller(&mut self) -> &mut RenderController;
}

impl RenderContext for RenderController {
    fn controller(&mut self) -> &mut RenderController {
        self
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
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

    pub fn translate_self(mut self, dx: f32, dy: f32) -> Self {
        self.translation.x += dx;
        self.translation.y += dy;
        self
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

impl Quad {
    pub fn triangles(&self) -> [Triangle; 2] {
        [
            Triangle {
                points: [
                    self.points[0].clone(),
                    self.points[1].clone(),
                    self.points[2].clone(),
                ],
            },
            Triangle {
                points: [
                    self.points[2].clone(),
                    self.points[3].clone(),
                    self.points[0].clone(),
                ],
            },
        ]
    }

    pub fn from_corner<P>(
        mut bottom_left: InputVertex,
        uv: Vec4,
        size: (f32, f32),
        mut positioner: P,
    ) -> Self
    where
        P: FnMut(&mut InputVertex, (f32, f32)),
    {
        let texture = bottom_left.texture;
        let has_texture = bottom_left.has_texture;
        let color = bottom_left.color;

        let vertex = || -> InputVertex {
            InputVertex {
                transform: Transform::new(),
                pos: (0.0, 0.0, f32::INFINITY),
                color: color.clone(),
                uv: (0.0, 0.0),
                texture,
                has_texture,
            }
        };

        let x1 = bottom_left.pos.0;
        let y1 = bottom_left.pos.1;
        let x2 = x1 + size.0;
        let y2 = y1 + size.1;

        let mut tl = vertex();
        let mut tr = vertex();
        let mut br = vertex();

        positioner(&mut bottom_left, (x1, y1));
        positioner(&mut tl, (x1, y2));
        positioner(&mut tr, (x2, y2));
        positioner(&mut br, (x2, y1));

        bottom_left.uv = (uv.x, uv.y - uv.w);
        tl.uv = (uv.x, uv.y);
        tr.uv = (uv.x + uv.z, uv.y);
        br.uv = (uv.x + uv.z, uv.y - uv.w);

        Self {
            points: [bottom_left, tl, tr, br],
        }
    }
}

pub trait PrimitiveRenderer {
    fn begin_frame(&mut self);
    fn end_frame(&mut self);
    fn begin_frame_to_target(&mut self, post: &mut RenderTarget);
    fn end_frame_to_target(&mut self, post: &mut RenderTarget);
    fn draw_data(
        &mut self,
        window: &Window,
        camera: &OrthographicCamera,
        vertices: &[u8],
        indices: &[u32],
        textures: &[GLuint],
        vbo: GLuint,
        ibo: GLuint,
        amount: u32,
        amount_textures: usize,
        shader: &mut OpenGLShader,
    );
    fn draw_data_to_target(
        &mut self,
        window: &Window,
        camera: &OrthographicCamera,
        vertices: &[u8],
        indices: &[u32],
        textures: &[GLuint],
        vbo: GLuint,
        ibo: GLuint,
        amount: u32,
        amount_textures: usize,
        shader: &mut OpenGLShader,
        post: &mut RenderTarget,
    );
}

pub struct OpenGLRenderer {
    framebuffer: GLuint,
    offscreen_target_1: GLuint,
    offscreen_target_2: GLuint,
    renderbuffer: GLuint,
    depth_texture: GLuint,
}

impl OpenGLRenderer {
    pub unsafe fn prepare(window: &Window) {
        let handle = window.get_handle();
        handle
            .make_current()
            .expect("Cannot make OpenGL context current");
    }

    pub unsafe fn initialize(window: &Window) -> Self {
        let mut offscreen_target_1 = 0;
        gl::GenTextures(1, &mut offscreen_target_1);
        gl::BindTexture(gl::TEXTURE_2D, offscreen_target_1);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            window.info.width as GLsizei,
            window.info.height as GLsizei,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            null(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        let mut offscreen_target_2 = 0;
        gl::GenTextures(1, &mut offscreen_target_2);
        gl::BindTexture(gl::TEXTURE_2D, offscreen_target_2);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            window.info.width as GLsizei,
            window.info.height as GLsizei,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            null(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        let mut fb = 0;
        gl::GenFramebuffers(1, &mut fb);
        gl::BindFramebuffer(gl::FRAMEBUFFER, fb);
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            offscreen_target_1,
            0,
        );

        let mut rb = 0;
        gl::GenRenderbuffers(1, &mut rb);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rb);
        gl::RenderbufferStorage(
            gl::RENDERBUFFER,
            gl::DEPTH_COMPONENT,
            window.info().width as GLsizei,
            window.info().height as GLsizei,
        );
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, rb);

        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);

        let attachments = [gl::COLOR_ATTACHMENT0];
        gl::DrawBuffers(1, attachments.as_ptr());

        let mut depth_texture = 0;
        gl::GenTextures(1, &mut depth_texture);
        gl::BindTexture(gl::TEXTURE_2D, depth_texture);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::DEPTH_COMPONENT24 as GLint,
            window.info.width as GLsizei,
            window.info.height as GLsizei,
            0,
            gl::DEPTH_COMPONENT,
            gl::FLOAT,
            null(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::DEPTH_ATTACHMENT,
            gl::TEXTURE_2D,
            depth_texture,
            0,
        );

        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        gl::Viewport(
            0,
            0,
            window.info().width as GLsizei,
            window.info().height as GLsizei,
        );

        Self {
            framebuffer: fb,
            offscreen_target_1,
            offscreen_target_2,
            renderbuffer: rb,
            depth_texture,
        }
    }

    pub fn clear() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
}

impl PrimitiveRenderer for OpenGLRenderer {
    fn begin_frame(&mut self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            gl::DepthMask(gl::TRUE);
            gl::DepthFunc(gl::ALWAYS);
        }
    }

    fn end_frame(&mut self) {}

    fn begin_frame_to_target(&mut self, post: &mut RenderTarget) {
        post.framebuffer = self.framebuffer;
        post.texture_1 = self.offscreen_target_1;
        post.texture_2 = self.offscreen_target_2;
        post.renderbuffer = self.renderbuffer;
        post.depth_texture = self.depth_texture;
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                self.offscreen_target_2,
                0,
            );
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::DepthMask(gl::TRUE);
            gl::DepthFunc(gl::ALWAYS);
        }
    }

    fn end_frame_to_target(&mut self, post: &mut RenderTarget) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    fn draw_data(
        &mut self,
        window: &Window,
        camera: &OrthographicCamera,
        vertices: &[u8],
        indices: &[u32],
        textures: &[GLuint],
        vbo: GLuint,
        ibo: GLuint,
        amount: u32,
        amount_textures: usize,
        shader: &mut OpenGLShader,
    ) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                vertices.len() as GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                indices.len() as GLsizeiptr * 4,
                indices.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );

            shader.uniform_1f("uResX", window.info.width as f32);
            shader.uniform_1f("uResY", window.info.height as f32);
            shader.uniform_matrix_4fv("uProjection", &camera.get_projection());
            shader.uniform_matrix_4fv("uView", &camera.get_view());

            for (i, texture) in textures.iter().take(amount_textures).enumerate() {
                gl::ActiveTexture(gl::TEXTURE0 + i as GLenum);
                gl::BindTexture(gl::TEXTURE_2D, *texture);
            }

            shader.uniform_1i("TEX_SAMPLER_0", 0);
            shader.uniform_1i("TEX_SAMPLER_1", 1);
            shader.uniform_1i("TEX_SAMPLER_2", 2);
            shader.uniform_1i("TEX_SAMPLER_3", 3);
            shader.uniform_1i("TEX_SAMPLER_4", 4);
            shader.uniform_1i("TEX_SAMPLER_5", 5);
            shader.uniform_1i("TEX_SAMPLER_6", 6);
            shader.uniform_1i("TEX_SAMPLER_7", 7);
            shader.uniform_1i("TEX_SAMPLER_8", 8);
            shader.uniform_1i("TEX_SAMPLER_9", 9);
            shader.uniform_1i("TEX_SAMPLER_10", 10);
            shader.uniform_1i("TEX_SAMPLER_11", 11);
            shader.uniform_1i("TEX_SAMPLER_12", 12);
            shader.uniform_1i("TEX_SAMPLER_13", 13);
            shader.uniform_1i("TEX_SAMPLER_14", 14);
            shader.uniform_1i("TEX_SAMPLER_15", 15);

            let stride = batch::VERTEX_SIZE_BYTES as GLsizei;

            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, transform.translation) as *const c_void,
            );
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, transform.origin) as *const c_void,
            );
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, transform.scale) as *const c_void,
            );
            gl::VertexAttribPointer(
                3,
                1,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, transform.rotation) as *const c_void,
            );

            gl::VertexAttribPointer(
                4,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, pos) as *const c_void,
            );
            gl::VertexAttribPointer(
                5,
                4,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, color) as *const c_void,
            );
            gl::VertexAttribPointer(
                6,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, uv) as *const c_void,
            );
            gl::VertexAttribPointer(
                7,
                1,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, texture) as *const c_void,
            );
            gl::VertexAttribPointer(
                8,
                1,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, has_texture) as *const c_void,
            );

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

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    fn draw_data_to_target(
        &mut self,
        window: &Window,
        camera: &OrthographicCamera,
        vertices: &[u8],
        indices: &[u32],
        textures: &[GLuint],
        vbo: GLuint,
        ibo: GLuint,
        amount: u32,
        amount_textures: usize,
        shader: &mut OpenGLShader,
        post: &mut RenderTarget,
    ) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                vertices.len() as GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                indices.len() as GLsizeiptr * 4,
                indices.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );

            shader.uniform_1f("uResX", window.info.width as f32);
            shader.uniform_1f("uResY", window.info.height as f32);
            shader.uniform_matrix_4fv("uProjection", &camera.get_projection());
            shader.uniform_matrix_4fv("uView", &camera.get_view());

            for (i, texture) in textures.iter().take(amount_textures).enumerate() {
                gl::ActiveTexture(gl::TEXTURE0 + i as GLenum);
                gl::BindTexture(gl::TEXTURE_2D, *texture);
            }

            shader.uniform_1i("TEX_SAMPLER_0", 0);
            shader.uniform_1i("TEX_SAMPLER_1", 1);
            shader.uniform_1i("TEX_SAMPLER_2", 2);
            shader.uniform_1i("TEX_SAMPLER_3", 3);
            shader.uniform_1i("TEX_SAMPLER_4", 4);
            shader.uniform_1i("TEX_SAMPLER_5", 5);
            shader.uniform_1i("TEX_SAMPLER_6", 6);
            shader.uniform_1i("TEX_SAMPLER_7", 7);
            shader.uniform_1i("TEX_SAMPLER_8", 8);
            shader.uniform_1i("TEX_SAMPLER_9", 9);
            shader.uniform_1i("TEX_SAMPLER_10", 10);
            shader.uniform_1i("TEX_SAMPLER_11", 11);
            shader.uniform_1i("TEX_SAMPLER_12", 12);
            shader.uniform_1i("TEX_SAMPLER_13", 13);
            shader.uniform_1i("TEX_SAMPLER_14", 14);
            shader.uniform_1i("TEX_SAMPLER_15", 15);

            let stride = batch::VERTEX_SIZE_BYTES as GLsizei;

            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, transform.translation) as *const c_void,
            );
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, transform.origin) as *const c_void,
            );
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, transform.scale) as *const c_void,
            );
            gl::VertexAttribPointer(
                3,
                1,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, transform.rotation) as *const c_void,
            );

            gl::VertexAttribPointer(
                4,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, pos) as *const c_void,
            );
            gl::VertexAttribPointer(
                5,
                4,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, color) as *const c_void,
            );
            gl::VertexAttribPointer(
                6,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, uv) as *const c_void,
            );
            gl::VertexAttribPointer(
                7,
                1,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, texture) as *const c_void,
            );
            gl::VertexAttribPointer(
                8,
                1,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, has_texture) as *const c_void,
            );

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

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        post.swap();
    }
}

impl Drop for OpenGLRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteRenderbuffers(1, &self.renderbuffer);
            gl::DeleteFramebuffers(1, &self.framebuffer);
            gl::DeleteTextures(1, &self.offscreen_target_1);
            gl::DeleteTextures(1, &self.offscreen_target_2);
            gl::DeleteTextures(1, &self.depth_texture);
        }
    }
}
