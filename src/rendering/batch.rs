use crate::rendering::camera::OrthographicCamera;
use crate::rendering::post::RenderTarget;
use crate::rendering::shader::OpenGLShader;
use crate::rendering::{PrimitiveRenderer, Quad, Triangle, Vertex};
use crate::window::Window;
use gl::types::{GLuint, GLuint64};

pub const BATCH_VERTEX_AMOUNT: usize = 100_000;

pub const VERTEX_SIZE_BYTES: usize = size_of::<Vertex>();
pub const VERTEX_SIZE: usize = VERTEX_SIZE_BYTES / 4;

pub const MAX_TEXTURES: usize = 16;

pub(crate) struct RenderBatch {
    pub(crate) vertex_data: Vec<u8>, // VERTEX_SIZE_BYTES * BATCH_VERTEX_AMOUNT
    pub(crate) index_data: Vec<u32>, // BATCH_VERTEX_AMOUNT * 6
    pub(crate) texture_data: [GLuint; MAX_TEXTURES],
    vertex_data_index: usize,
    vertex_index: usize,
    index_index: usize,
    texture_index: usize,
    triangle_index: usize,
    vbo_id: GLuint,
    ibo_id: GLuint,
    shader: GLuint
}

impl RenderBatch {
    pub(crate) unsafe fn new(shader: GLuint) -> Self {
        let mut vbo_id = 0;
        let mut ibo_id = 0;
        gl::GenBuffers(1, &mut vbo_id);
        gl::GenBuffers(1, &mut ibo_id);

        let mut texture_units = 0;
        gl::GetIntegerv(gl::MAX_TEXTURE_IMAGE_UNITS, &mut texture_units);

        Self {
            vertex_data: vec![0; VERTEX_SIZE_BYTES * BATCH_VERTEX_AMOUNT],
            index_data: vec![0; BATCH_VERTEX_AMOUNT * 6],
            texture_data: [0; MAX_TEXTURES],
            vertex_data_index: 0,
            vertex_index: 0,
            index_index: 0,
            texture_index: 0,
            triangle_index: 0,
            vbo_id,
            ibo_id,
            shader,
        }
    }

    pub(crate) fn push_triangle(&mut self, mut triangle: Triangle) {
        for (idx, vertex) in triangle.points.into_iter().enumerate() {
            let mut r_vertex = Vertex::from_inp(&vertex, 0.0);
            if r_vertex.has_texture == 1.0 {
                let req_id = vertex.texture;
                if let Some(idx) = self.texture_data.iter().position(|id| *id == req_id) {
                    r_vertex.texture = idx as f32;
                } else {
                    r_vertex.texture = self.texture_index as f32;
                    self.texture_data[self.texture_index] = req_id;
                    self.texture_index += 1;
                }
            }

            unsafe {
                let src_ptr = &r_vertex as *const Vertex as *const u8;
                let dst_ptr = self.vertex_data.as_mut_ptr().add(self.vertex_data_index) as *mut u8;

                std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, VERTEX_SIZE_BYTES);

                self.vertex_data_index += VERTEX_SIZE_BYTES;
            }
        }

        self.index_data[self.index_index + 0] = self.vertex_index as u32 + 0;
        self.index_data[self.index_index + 1] = self.vertex_index as u32 + 1;
        self.index_data[self.index_index + 2] = self.vertex_index as u32 + 2;

        self.index_index += 3;
        self.triangle_index += 1;
        self.vertex_index += 3;
    }

    pub(crate) fn push_quad(&mut self, mut quad: Quad) {
        for (idx, vertex) in quad.points.into_iter().enumerate() {
            let mut r_vertex = Vertex::from_inp(&vertex, 0.0);
            if r_vertex.has_texture == 1.0 {
                let req_id = vertex.texture;
                if let Some(idx) = self.texture_data.iter().position(|id| *id == req_id) {
                    r_vertex.texture = idx as f32;
                } else {
                    r_vertex.texture = self.texture_index as f32;
                    self.texture_data[self.texture_index] = req_id;
                    self.texture_index += 1;
                }
            }

            unsafe {
                let src_ptr = &r_vertex as *const Vertex as *const u8;
                let dst_ptr = self.vertex_data.as_mut_ptr().add(self.vertex_data_index) as *mut u8;

                std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, VERTEX_SIZE_BYTES);

                self.vertex_data_index += VERTEX_SIZE_BYTES;
            }
        }

        self.index_data[self.index_index + 0] = self.vertex_index as u32 + 0;
        self.index_data[self.index_index + 1] = self.vertex_index as u32 + 1;
        self.index_data[self.index_index + 2] = self.vertex_index as u32 + 2;

        self.index_data[self.index_index + 3] = self.vertex_index as u32 + 2;
        self.index_data[self.index_index + 4] = self.vertex_index as u32 + 3;
        self.index_data[self.index_index + 5] = self.vertex_index as u32 + 0;

        self.index_index += 6;
        self.triangle_index += 2;
        self.vertex_index += 4;
    }

    fn has_texture(&self, id: GLuint) -> bool {
        self.texture_data.contains(&id)
    }

    pub fn can_hold_triangle(&self, triangle: &Triangle) -> bool {
        if self.vertex_index + 3 > BATCH_VERTEX_AMOUNT { return false; }

        let mut needed_tex = 0;
        let mut seen = Vec::new();
        for vertex in &triangle.points {
            if vertex.has_texture == 1.0 {
                if !seen.contains(&vertex.texture) && !self.has_texture(vertex.texture) {
                    needed_tex += 1;
                    seen.push(vertex.texture);
                }
            }
        }

        if self.texture_index + needed_tex > MAX_TEXTURES {
            return false;
        }

        true
    }

    pub fn can_hold_quad(&self, quad: &Quad) -> bool {
        if self.vertex_index + 4 > BATCH_VERTEX_AMOUNT { return false; }

        let mut needed_tex = 0;
        let mut seen = Vec::new();
        for vertex in &quad.points {
            if vertex.has_texture == 1.0 {
                if !seen.contains(&vertex.texture) && !self.has_texture(vertex.texture) {
                    needed_tex += 1;
                    seen.push(vertex.texture);
                }
            }
        }

        if self.texture_index + needed_tex > MAX_TEXTURES {
            return false;
        }

        true
    }

    pub(crate) fn prepare_batch(&mut self) {
        self.vertex_data_index = 0;
        self.vertex_index = 0;
        self.index_index = 0;
        self.triangle_index = 0;
        self.texture_index = 0;
        self.texture_data.fill(0);
    }

    pub fn is_empty(&self) -> bool {
        self.vertex_data_index == 0
    }

    pub fn draw(&mut self, window: &Window, camera: &OrthographicCamera, renderer: &mut impl PrimitiveRenderer, shader: &mut OpenGLShader) {
        renderer.draw_data(
            window,
            camera,
            &self.vertex_data,
            &self.index_data,
            &self.texture_data,
            self.vbo_id,
            self.ibo_id,
            self.triangle_index as u32 * 3,
            self.texture_index,
            shader
        );
        self.prepare_batch();
    }

    pub fn draw_to_target(&mut self, window: &Window, camera: &OrthographicCamera, renderer: &mut impl PrimitiveRenderer, shader: &mut OpenGLShader, post: &mut RenderTarget) {
        renderer.draw_data_to_target(
            window,
            camera,
            &self.vertex_data,
            &self.index_data,
            &self.texture_data,
            self.vbo_id,
            self.ibo_id,
            self.triangle_index as u32 * 3,
            self.texture_index,
            shader,
            post
        );
        self.prepare_batch();
    }
}

impl Drop for RenderBatch {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo_id);
            gl::DeleteBuffers(1, &self.ibo_id);
        }
    }
}