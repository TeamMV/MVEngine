use std::cell::RefCell;
use std::ops::Deref;
use std::os::raw::c_void;

use alloc::rc::Rc;
use gl::types::{GLenum, GLint, GLsizei, GLsizeiptr, GLuint};
use mvutils::utils::RcMut;

use crate::old_render::batch3d::batch_layout_3d;
use crate::old_render::camera::{Camera2D, Camera3D};
use crate::old_render::lights::Light;
use crate::old_render::model::Model;
use crate::old_render::opengl::opengl::gen_buffer_id;
use crate::old_render::shared::{EffectShader, RenderProcessor3D, Shader, Texture};

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
        gl::DrawBuffers(3, attach.as_ptr());

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            panic!("Incomplete 3D Framebuffer!");
        }

        gl::BindTexture(gl::TEXTURE_2D, 0);
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
}

pub(crate) struct OpenGLGeometryPass {
    pub(crate) buffer: GBuffer,
    width: i32,
    height: i32,
    vbo: u32,
    ibo: u32,
    camera: Option<Camera3D>,
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
        unsafe { self.buffer.generate(width, height); }
    }

    pub(super) fn resize(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    pub(super) fn set_camera(&mut self, cam: Camera3D) {
        self.camera = Some(cam);
    }
}

macro_rules! vert_attrib {
    ($idx:expr, $size:ident, $off:ident) => {
        gl::VertexAttribPointer($idx, batch_layout_3d::$size as GLint, gl::FLOAT, 0, batch_layout_3d::MODEL_VERTEX_SIZE_BYTES as GLsizei, batch_layout_3d::$off as *const _);
        gl::EnableVertexAttribArray($idx);
    };
}

impl RenderProcessor3D for OpenGLGeometryPass {
    #[allow(clippy::too_many_arguments)]
    fn process_batch(&self, tex: &mut [Option<Rc<RefCell<Texture>>>], tex_id: &[u32], indices: &[u32], vertices: &[f32], shader: &mut Shader, render_mode: u8) {
        let mut i: u8 = 0;
        for t in tex.iter_mut().flatten() {
            t.borrow_mut().bind(i);
            i += 1;
        }

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * 4) as GLsizeiptr, vertices.as_ptr() as *const c_void, gl::DYNAMIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (indices.len() * 4) as GLsizeiptr, indices.as_ptr() as *const c_void, gl::DYNAMIC_DRAW);

            gl::BindFramebuffer(gl::FRAMEBUFFER, self.buffer.id);

            vert_attrib!(0, POSITION_SIZE, POSITION_OFFSET_BYTES);
            vert_attrib!(1, NORMAL_SIZE, NORMAL_OFFSET_BYTES);
            vert_attrib!(2, UV_SIZE, UV_OFFSET_BYTES);
            vert_attrib!(3, MATERIAL_ID_SIZE, MATERIAL_ID_OFFSET_BYTES);
            vert_attrib!(4, CANVAS_COORDS_SIZE, CANVAS_COORDS_OFFSET_BYTES);
            vert_attrib!(5, CANVAS_DATA_SIZE, CANVAS_DATA_OFFSET_BYTES);
            vert_attrib!(6, MODEL_MATRIX_SIZE, MODEL_MATRIX_OFFSET_BYTES);

            gl::DrawElements(render_mode as GLenum, indices.len() as GLsizei, gl::UNSIGNED_INT, std::ptr::null());

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn process_model(&self, tex: &mut [Option<Rc<RefCell<Texture>>>], tex_id: &[u32], indices: &[u32], vertices: &[f32], shader: &mut Shader, render_mode: u8) {
        let mut i: u8 = 0;
        for t in tex.iter_mut().flatten() {
            t.borrow_mut().bind(i);
            i += 1;
        }

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * 4) as GLsizeiptr, vertices.as_ptr() as *const c_void, gl::DYNAMIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (indices.len() * 4) as GLsizeiptr, indices.as_ptr() as *const c_void, gl::DYNAMIC_DRAW);

            gl::BindFramebuffer(gl::FRAMEBUFFER, self.buffer.id);

            vert_attrib!(0, POSITION_SIZE, POSITION_OFFSET_BYTES);
            vert_attrib!(1, NORMAL_SIZE, NORMAL_OFFSET_BYTES);
            vert_attrib!(2, UV_SIZE, UV_OFFSET_BYTES);
            vert_attrib!(3, MATERIAL_ID_SIZE, MATERIAL_ID_OFFSET_BYTES);

            gl::DrawElements(render_mode as GLenum, indices.len() as GLsizei, gl::UNSIGNED_INT, std::ptr::null());

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }
}

pub(crate) struct OpenGLLightingPass {
    quad_indices: [u32; 6],
    ibo: GLuint,
}

impl OpenGLLightingPass {
    pub(super) fn new() -> Self {
        OpenGLLightingPass {
            quad_indices: [0, 2, 1, 1, 2, 3],
            ibo: 0,
        }
    }

    pub(super) fn setup(&mut self) {
        self.ibo = gen_buffer_id();
    }

    pub(crate) fn light_scene(&self, deferred_shader: Rc<RefCell<EffectShader>>, gbuffer: &GBuffer, camera: &Camera3D, lights: &[Light]) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, 24 as GLsizeiptr, self.quad_indices.as_ptr() as *const c_void, gl::STATIC_DRAW);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, gbuffer.albedo);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, gbuffer.normals);

            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, gbuffer.positions);

            deferred_shader.borrow_mut().bind();

            deferred_shader.borrow_mut().uniform_1i("gAlbedoSpec", 0);
            deferred_shader.borrow_mut().uniform_1i("gNormals", 1);
            deferred_shader.borrow_mut().uniform_1i("gPosition", 2);
            deferred_shader.borrow_mut().uniform_1i("numLights", lights.len() as i32);

            for (i, l) in lights.iter().enumerate() {
                deferred_shader.borrow_mut().uniform_light(format!("lights[{}]", i).as_str(), l);
            }

            deferred_shader.borrow_mut().uniform_3fv("viewPos", camera.deref().position);

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, 0);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, 0);

            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}