use alloc::rc::Rc;
use std::cell::RefCell;
use gl::types::{GLint, GLuint};
use mvutils::utils::RcMut;
use crate::render::camera::Camera;
use crate::render::opengl::opengl::gen_buffer_id;
use crate::render::shared::{RenderProcessor3D, Shader, Texture};

pub(crate) struct GBuffer {
    width: i32,
    height: i32,
    id: GLuint,
    albedo: GLuint,
    normals: GLuint,
    positions: GLuint,
    depth: GLuint,
}

impl GBuffer {
    pub(crate) fn new() -> Self {
        GBuffer {
            width: 0,
            height: 0,
            id: 0,
            albedo: 0,
            normals: 0,
            positions: 0,
            depth: 0,
        }
    }

    pub(crate) unsafe fn generate(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;

        if self.id != 0 {
            gl::DeleteFramebuffers(1, &self.id);
        }
        if self.albedo != 0 {
            gl::DeleteTextures(1, &self.albedo);
        }
        if self.normals != 0 {
            gl::DeleteTextures(1, &self.normals);
        }
        if self.positions != 0 {
            gl::DeleteTextures(1, &self.positions);
        }
        if self.depth != 0 {
            gl::DeleteRenderbuffers(1, &self.depth);
        }

        gl::CreateFramebuffers(1, &mut self.id);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);

        gl::GenTextures(1, &mut self.albedo);
        gl::BindTexture(gl::TEXTURE_2D, self.albedo);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, self.width, self.height, 0, gl::RGBA, gl::UNSIGNED_BYTE, std::ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, self.albedo, 0);

        gl::GenTextures(1, &mut self.normals);
        gl::BindTexture(gl::TEXTURE_2D, self.normals);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA16F as GLint, self.width, self.height, 0, gl::RGBA, gl::FLOAT, std::ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::TEXTURE_2D, self.normals, 0);

        gl::GenTextures(1, &mut self.positions);
        gl::BindTexture(gl::TEXTURE_2D, self.positions);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA16F as GLint, self.width, self.height, 0, gl::RGBA, gl::FLOAT, std::ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT2, gl::TEXTURE_2D, self.positions, 0);

        gl::GenRenderbuffers(1, &mut self.depth);
        gl::BindRenderbuffer(gl::RENDERBUFFER, self.depth);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT, self.width, self.height);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, self.depth);

        let attach = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1, gl::COLOR_ATTACHMENT2];
        gl::DrawBuffers(1, attach.as_ptr());

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            panic!("Incomplete 3D Framebuffer!");
        }

        gl::BindTexture(gl::TEXTURE_2D, 0);
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);


    }
}

pub(crate) struct OpenGLGeometryPass {
    buffer: GBuffer,
    width: i32,
    height: i32,
    vbo: u32,
    ibo: u32,
    camera: Option<Camera>
}

impl OpenGLGeometryPass {
    pub(super) fn new() -> Self {
        OpenGLGeometryPass {
            buffer: GBuffer::new(),
            width: 0,
            height: 0,
            vbo: 0,
            ibo: 0,
            camera: None,
        }
    }

    pub(super) fn setup(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
        self.vbo = gen_buffer_id();
        self.ibo = gen_buffer_id();
    }

    pub(super) fn resize(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    pub(super) fn set_camera(&mut self, cam: Camera) {
        self.camera = Some(cam);
    }
}

impl RenderProcessor3D for OpenGLGeometryPass {
    #[allow(clippy::too_many_arguments)]
    fn process_batch(&self, tex: &mut [Option<Rc<RefCell<Texture>>>], tex_id: &[u32], indices: &[u32], vertices: &[f32], shader: &mut Shader, render_mode: u8) {
        todo!()
    }

    #[allow(clippy::too_many_arguments)]
    fn process_model(&self, tex: &mut [Option<Rc<RefCell<Texture>>>], tex_id: &[u32], indices: &[u32], vertices: &[f32], shader: &mut Shader, render_mode: u8) {
        todo!()
    }
}

pub(crate) struct OpenGLLightingPass {

}

impl OpenGLLightingPass {
    pub(super) fn new() -> Self {
        OpenGLLightingPass {

        }
    }
}