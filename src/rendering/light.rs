use crate::math::vec::{Vec2, Vec4};
use crate::rendering::camera::OrthographicCamera;
use crate::rendering::post::RenderTarget;
use crate::rendering::shader::OpenGLShader;
use crate::rendering::{batch, bindless, PrimitiveRenderer, Vertex};
use crate::window::Window;
use gl::types::{GLenum, GLint, GLsizei, GLsizeiptr, GLuint, GLuint64};
use std::mem::offset_of;
use std::os::raw::c_void;
use std::ptr::null;
use itertools::Itertools;
use crate::color::RgbColor;

#[repr(C)]
#[derive(Clone)]
pub struct Light {
    pub pos: Vec2,
    pub color: Vec4,
    pub intensity: f32,
    pub range: f32,   // Maximum range of the light
    pub falloff: f32, // How sharply the intensity decays
}

pub struct LightOpenGLRenderer {
    ambient: Vec4,
    lights: Vec<Light>,
    framebuffer: GLuint,
    offscreen_target_1: GLuint,
    offscreen_target_2: GLuint,
    renderbuffer: GLuint,
    depth_texture: GLuint
}

impl LightOpenGLRenderer {
    pub unsafe fn prepare(window: &Window) {
        let handle = window.get_handle();
        handle.make_current().expect("Cannot make OpenGL context current");
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
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, offscreen_target_1, 0);

        let mut rb = 0;
        gl::GenRenderbuffers(1, &mut rb);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rb);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT, window.info().width as GLsizei, window.info().height as GLsizei);
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

        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, depth_texture, 0);

        gl::Enable(gl::DEPTH_TEST);

        gl::Viewport(0, 0, window.info().width as GLsizei, window.info().height as GLsizei);

        Self {
            ambient: RgbColor::new([50, 50, 50, 255]).as_vec4(),
            lights: vec![],
            framebuffer: fb,
            offscreen_target_1,
            offscreen_target_2,
            renderbuffer: rb,
            depth_texture,
        }
    }



    pub fn push_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub fn lights(&self) -> &Vec<Light> {
        &self.lights
    }

    pub fn lights_mut(&mut self) -> &mut Vec<Light> {
        &mut self.lights
    }

    pub fn ambient(&self) -> Vec4 {
        self.ambient
    }

    pub fn set_ambient(&mut self, ambient: Vec4) {
        self.ambient = ambient;
    }
}

impl PrimitiveRenderer for LightOpenGLRenderer {
    fn begin_frame(&mut self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            gl::DepthMask(gl::TRUE);
            gl::DepthFunc(gl::ALWAYS);

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    fn end_frame(&mut self) {

    }

    fn begin_frame_to_target(&mut self, post: &mut RenderTarget) {
        post.framebuffer = self.framebuffer;
        post.texture_1 = self.offscreen_target_1;
        post.texture_2 = self.offscreen_target_2;
        post.renderbuffer = self.renderbuffer;
        post.depth_texture = self.depth_texture;
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, self.offscreen_target_2, 0);
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

    fn draw_data(&mut self, window: &Window, camera: &OrthographicCamera, vertices: &[u8], indices: &[u32], textures: &[GLuint], vbo: GLuint, ibo: GLuint, amount: u32, amount_textures: usize, shader: &mut OpenGLShader) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, vertices.len() as GLsizeiptr, vertices.as_ptr() as *const _, gl::DYNAMIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, indices.len() as GLsizeiptr * 4, indices.as_ptr() as *const _, gl::DYNAMIC_DRAW);

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

            shader.uniform_1i("NUM_LIGHTS", self.lights.len() as i32);

            for (i, light) in self.lights.iter().enumerate().take(amount_textures) {
                let index = i as i32;
                let light_name = format!("LIGHTS[{}]", index);

                shader.uniform_2fv(&format!("{}.pos", light_name), &light.pos);
                shader.uniform_4fv(&format!("{}.color", light_name), &light.color);
                shader.uniform_1f(&format!("{}.intensity", light_name), light.intensity);
                shader.uniform_1f(&format!("{}.range", light_name), light.range);
                shader.uniform_1f(&format!("{}.falloff", light_name), light.falloff);
            }

            shader.uniform_4fv("AMBIENT", &self.ambient);

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

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    fn draw_data_to_target(&mut self, window: &Window, camera: &OrthographicCamera, vertices: &[u8], indices: &[u32], textures: &[GLuint], vbo: GLuint, ibo: GLuint, amount: u32, amount_textures: usize, shader: &mut OpenGLShader, post: &mut RenderTarget) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, vertices.len() as GLsizeiptr, vertices.as_ptr() as *const _, gl::DYNAMIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, indices.len() as GLsizeiptr * 4, indices.as_ptr() as *const _, gl::DYNAMIC_DRAW);

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

            shader.uniform_1i("NUM_LIGHTS", self.lights.len() as i32);

            for (i, light) in self.lights.iter().enumerate() {
                let index = i as i32;
                let light_name = format!("LIGHTS[{}]", index);

                shader.uniform_2fv(&format!("{}.pos", light_name), &light.pos);
                shader.uniform_4fv(&format!("{}.color", light_name), &light.color);
                shader.uniform_1f(&format!("{}.intensity", light_name), light.intensity);
                shader.uniform_1f(&format!("{}.range", light_name), light.range);
                shader.uniform_1f(&format!("{}.falloff", light_name), light.falloff);
            }

            shader.uniform_4fv("AMBIENT", &self.ambient);

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

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        post.swap();
    }
}

impl Drop for LightOpenGLRenderer {
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

