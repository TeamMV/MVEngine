use crate::math::vec::Vec2;
use crate::rendering::backbuffer::{BackBuffer, BackBufferTarget};
use crate::rendering::shader::OpenGLShader;
use crate::rendering::texture::Texture;
use gl::types::{GLenum, GLsizei, GLsizeiptr, GLuint};
use std::ops::{Deref, DerefMut};
use std::os::raw::c_void;
use std::ptr::null;
use std::sync::atomic::{AtomicU8, Ordering};

const RESERVED_TEXTURES: u8 = 2;

pub struct OpenGLPostProcessShader {
    inner: OpenGLShader,
    tex_count: AtomicU8,
}

impl Clone for OpenGLPostProcessShader {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            tex_count: AtomicU8::new(0),
        }
    }
}

impl OpenGLPostProcessShader {
    pub fn new(fragment_code: &'static str) -> Self {
        let inner = OpenGLShader::new(
            include_str!("shaders/screen.vert"),
            fragment_code,
        );

        Self{
            inner,
            tex_count: AtomicU8::new(0),
        }
    }

    pub fn uniform_texture(&self, texture: &Texture, name: &str) {
        let tex_count = self.tex_count.load(Ordering::Acquire);
        let id = RESERVED_TEXTURES + tex_count;
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + id as GLenum);
            gl::BindTexture(gl::TEXTURE_2D, texture.id);
            self.uniform_1i(name, id as i32);
        }
        self.tex_count.store(tex_count + 1, Ordering::Release);
    }

    pub fn begin_frame(&self) {
        self.tex_count.store(0, Ordering::Release);
    }
}

impl Deref for OpenGLPostProcessShader {
    type Target = OpenGLShader;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for OpenGLPostProcessShader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// A special fragment shader with 2 inputs: SRC and DST (both textures). DST is the backbuffer content and SRC is the currently rendered quad.
pub type OpenGlBlendShader = OpenGLPostProcessShader;

#[derive(Clone)]
pub struct RenderTarget {
    pub(crate) texture_1: GLuint,
    pub(crate) texture_2: GLuint,
    pub(crate) framebuffer: GLuint,
    pub(crate) renderbuffer: GLuint,
    pub(crate) depth_texture: GLuint,
}

impl RenderTarget {
    pub fn from_backbuffer(bb: &BackBuffer) -> Self {
        Self {
            texture_1: bb.tex,
            texture_2: bb.swap_tex,
            framebuffer: bb.fbo,
            renderbuffer: bb.rbo,
            depth_texture: bb.depth_tex,
        }
    }
    
    pub fn swap(&mut self) {
        let tmp = self.texture_1;
        self.texture_1 = self.texture_2;
        self.texture_2 = tmp;
    }
}

pub struct OpenGLPostProcessRenderer {
    vbo: GLuint,
    ibo: GLuint,
    screen_vertex_data: [f32; 16],
    screen_index_data: [u32; 6],
    screen_shader: OpenGLPostProcessShader,
    target: RenderTarget,
    res: Vec2,
}

impl OpenGLPostProcessRenderer {
    pub fn new(width: i32, height: i32) -> Self {
        unsafe {
            let mut vbo_id = 0;
            let mut ibo_id = 0;
            gl::GenBuffers(1, &mut vbo_id);
            gl::GenBuffers(1, &mut ibo_id);

            let mut screen_shader =
                OpenGLPostProcessShader::new(include_str!("shaders/screen.frag"));
            screen_shader.make().expect("invalid mve shader");
            screen_shader.bind().expect("invalid mve shader");

            Self {
                vbo: vbo_id,
                ibo: ibo_id,
                screen_vertex_data: [
                    -1.0, -1.0, 0.0, 0.0, 1.0, -1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, 0.0,
                    1.0,
                ],
                screen_index_data: [0, 1, 2, 2, 3, 0],
                screen_shader,
                target: RenderTarget {
                    texture_1: 0,
                    texture_2: 0,
                    framebuffer: 0,
                    renderbuffer: 0,
                    depth_texture: 0,
                },
                res: Vec2::new(width as f32, height as f32),
            }
        }
    }

    pub fn set_target(&mut self, target: RenderTarget) {
        self.target = target;
    }

    pub fn run_shader(&mut self, shader: &mut OpenGLPostProcessShader) {
        unsafe {
            shader.begin_frame();
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.target.texture_1);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, self.target.depth_texture);

            gl::DepthMask(gl::FALSE);
            gl::DepthFunc(gl::ALWAYS);

            shader.uniform_1i("COLOR", 0);
            shader.uniform_1i("DEPTH", 1);
            shader.uniform_2fv("RES", &self.res);

            gl::BindFramebuffer(gl::FRAMEBUFFER, self.target.framebuffer);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                self.target.texture_2,
                0,
            );
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                self.screen_vertex_data.len() as GLsizeiptr * 4,
                self.screen_vertex_data.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                self.screen_index_data.len() as GLsizeiptr * 4,
                self.screen_index_data.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );

            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 4 * 4, 0 as *const c_void);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 4 * 4, 8 as *const c_void);

            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);

            gl::DrawElements(gl::TRIANGLES, 6 as GLsizei, gl::UNSIGNED_INT, null());

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        self.target.swap();
    }

    pub fn draw_to_screen(&self) {
        self.draw_to_backbuffer(&mut BackBufferTarget::Screen);
    }

    pub fn draw_to_backbuffer(&self, target: &mut BackBufferTarget) {
        unsafe {
            self.screen_shader.use_program();
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.target.texture_1);

            self.screen_shader.uniform_1i("COLOR", 0);

            target.bind();
            //gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            //gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::DepthMask(gl::FALSE);
            gl::DepthFunc(gl::ALWAYS);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                self.screen_vertex_data.len() as GLsizeiptr * 4,
                self.screen_vertex_data.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                self.screen_index_data.len() as GLsizeiptr * 4,
                self.screen_index_data.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );

            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 4 * 4, 0 as *const c_void);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 4 * 4, 8 as *const c_void);

            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);

            gl::DrawElements(gl::TRIANGLES, 6 as GLsizei, gl::UNSIGNED_INT, null());

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

            gl::BindTexture(gl::TEXTURE_2D, 0);
            target.unbind();
        }
    }

    pub fn draw_to_backbuffer_custom_blend_shader(&self, target: &mut BackBufferTarget, blend_shader: &OpenGlBlendShader) {
        unsafe {
            blend_shader.use_program();
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.target.texture_1);
            if let BackBufferTarget::Buffer(bb) = target {
                gl::ActiveTexture(gl::TEXTURE1);
                gl::BindTexture(gl::TEXTURE_2D, bb.swap_tex);
            }

            blend_shader.uniform_1i("SRC", 0);
            blend_shader.uniform_1i("DST", 1);

            target.bind();
            //gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            //gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::DepthMask(gl::FALSE);
            gl::DepthFunc(gl::ALWAYS);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                self.screen_vertex_data.len() as GLsizeiptr * 4,
                self.screen_vertex_data.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                self.screen_index_data.len() as GLsizeiptr * 4,
                self.screen_index_data.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );

            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 4 * 4, 0 as *const c_void);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 4 * 4, 8 as *const c_void);

            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);

            gl::DrawElements(gl::TRIANGLES, 6 as GLsizei, gl::UNSIGNED_INT, null());

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

            gl::BindTexture(gl::TEXTURE_2D, 0);
            target.unbind();
        }
    }

    pub fn target(&self) -> RenderTarget {
        self.target.clone()
    }
}

impl Drop for OpenGLPostProcessRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ibo);
        }
    }
}
