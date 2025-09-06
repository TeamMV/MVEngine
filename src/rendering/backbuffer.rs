use std::mem;
use std::sync::atomic::Ordering;
use gl::types::GLuint;
use log::{debug, trace};
use crate::rendering::{OpenGLRenderer, CLEAR_FLAG};

#[derive(Clone)]
pub enum BackBufferTarget {
    Screen,
    Buffer(BackBuffer)
}

impl BackBufferTarget {
    pub fn bind(&self) {
        unsafe {
            if let Self::Buffer(buffer) = self {
                gl::BindFramebuffer(gl::FRAMEBUFFER, buffer.fbo);
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    buffer.swap_tex,
                    0,
                );
                if CLEAR_FLAG.load(Ordering::Acquire) {
                    trace!("Cleared backbuffer");
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                }
            } else {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            }
        }
    }

    pub fn unbind(&mut self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            //self.swap();
        }
    }

    pub fn swap(&mut self) {
        if let BackBufferTarget::Buffer(bb) = self {
            bb.swap();
        }
    }
}

pub struct BackBuffer {
    pub rbo: GLuint,
    pub fbo: GLuint,
    pub tex: GLuint,
    pub swap_tex: GLuint,
    pub depth_tex: GLuint,
    is_copy: bool,
}

impl BackBuffer {
    pub fn new(width: i32, height: i32) -> Self {
        unsafe {
            let tex = OpenGLRenderer::create_texture_rgba(width, height);
            let swap_tex = OpenGLRenderer::create_texture_rgba(width, height);

            let fbo = OpenGLRenderer::create_framebuffer_with_color(tex);

            let rbo = OpenGLRenderer::create_depth_renderbuffer(width, height);
            OpenGLRenderer::attach_renderbuffer_to_framebuffer(fbo, rbo);

            let depth_tex = OpenGLRenderer::create_texture_depth(width, height);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                depth_tex,
                0,
            );

            let attachments = [gl::COLOR_ATTACHMENT0];
            gl::DrawBuffers(1, attachments.as_ptr());

            BackBuffer {
                rbo,
                fbo,
                tex,
                swap_tex,
                depth_tex,
                is_copy: false,
            }
        }
    }

    pub fn swap(&mut self) {
        mem::swap(&mut self.tex, &mut self.swap_tex);
    }
}

impl Clone for BackBuffer {
    fn clone(&self) -> Self {
        Self {
            rbo: self.rbo,
            fbo: self.fbo,
            tex: self.tex,
            swap_tex: self.swap_tex,
            depth_tex: self.depth_tex,
            is_copy: true,
        }
    }
}

impl Drop for BackBuffer {
    fn drop(&mut self) {
        if !self.is_copy {
            unsafe {
                gl::DeleteRenderbuffers(1, &self.rbo);
                gl::DeleteFramebuffers(1, &self.fbo);
                gl::DeleteTextures(1, &self.tex);
                gl::DeleteTextures(1, &self.swap_tex);
                gl::DeleteTextures(1, &self.depth_tex);
            }
        }
    }
}