use crate::rendering::camera::OrthographicCamera;
use crate::rendering::post::RenderTarget;
use crate::rendering::shader::OpenGLShader;
use crate::rendering::{InputVertex, PrimitiveRenderer, Quad, Triangle, Vertex};
use crate::window::Window;
use gl::types::GLuint;
use crate::rendering::backbuffer::BackBufferTarget;

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
    _shader: GLuint,
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
            _shader: shader,
        }
    }

    pub(crate) fn push_triangle(&mut self, triangle: Triangle) {
        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.render_batch(|t| t.resume());
        }
        for vertex in triangle.points.into_iter() {
            let mut r_vertex = Vertex::from_inp(&vertex, 0.0);
            if r_vertex.has_texture > 0.0 {
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

        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.render_batch(|t| t.pause());
        }
    }

    pub(crate) fn push_quad(&mut self, quad: Quad) {
        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.render_batch(|t| t.resume());
        }
        for vertex in quad.points.into_iter() {
            let mut r_vertex = Vertex::from_inp(&vertex, 0.0);
            if r_vertex.has_texture > 0.0 {
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

        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.render_batch(|t| t.pause());
        }
    }

    pub fn push_raw<F: Fn(&mut InputVertex)>(
        &mut self,
        vertices: &[InputVertex],
        indices: &[usize],
        modifier: Option<F>,
    ) {
        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.render_batch(|t| t.resume());
        }
        for mut vertex in vertices.to_vec() {
            if let Some(ref modify) = modifier {
                modify(&mut vertex);
            }

            let mut r_vertex = Vertex::from_inp(&vertex, 0.0);

            if r_vertex.has_texture > 0.0 {
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

        let base_index = self.vertex_index as u32;
        for &i in indices {
            self.index_data[self.index_index] = base_index + i as u32;
            self.index_index += 1;
        }

        self.triangle_index += indices.len() / 3;
        self.vertex_index += vertices.len();

        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.render_batch(|t| t.pause());
        }
    }

    fn has_texture(&self, id: GLuint) -> bool {
        self.texture_data.contains(&id)
    }

    pub fn can_hold_triangle(&self, triangle: &Triangle) -> bool {
        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.render_batch(|t| t.resume());
        }
        if self.vertex_index + 3 > BATCH_VERTEX_AMOUNT {
            #[cfg(feature = "timed")] {
                crate::debug::PROFILER.render_batch(|t| t.pause());
            }
            return false;
        }

        let mut needed_tex = 0;
        let mut seen = Vec::new();
        for vertex in &triangle.points {
            if vertex.has_texture > 0.0 {
                if !seen.contains(&vertex.texture) && !self.has_texture(vertex.texture) {
                    needed_tex += 1;
                    seen.push(vertex.texture);
                }
            }
        }

        if self.texture_index + needed_tex > MAX_TEXTURES {
            #[cfg(feature = "timed")] {
                crate::debug::PROFILER.render_batch(|t| t.pause());
            }
            
            return false;
        }

        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.render_batch(|t| t.pause());
        }

        true
    }

    pub fn can_hold_quad(&self, quad: &Quad) -> bool {
        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.render_batch(|t| t.resume());
        }
        if self.vertex_index + 4 > BATCH_VERTEX_AMOUNT {
            #[cfg(feature = "timed")] {
                crate::debug::PROFILER.render_batch(|t| t.pause());
            }
            return false;
        }

        let mut needed_tex = 0;
        let mut seen = Vec::new();
        for vertex in &quad.points {
            if vertex.has_texture > 0.0 {
                if !seen.contains(&vertex.texture) && !self.has_texture(vertex.texture) {
                    needed_tex += 1;
                    seen.push(vertex.texture);
                }
            }
        }

        if self.texture_index + needed_tex > MAX_TEXTURES {
            #[cfg(feature = "timed")] {
                crate::debug::PROFILER.render_batch(|t| t.pause());
            }
            return false;
        }

        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.render_batch(|t| t.pause());
        }

        true
    }

    pub fn can_hold_vertices(&self, vertices: &[InputVertex], has_tex: bool) -> bool {
        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.render_batch(|t| t.resume());
        }
        let len = vertices.len();
        if self.vertex_index + len > BATCH_VERTEX_AMOUNT {
            #[cfg(feature = "timed")] {
                crate::debug::PROFILER.render_batch(|t| t.pause());
            }
            return false;
        }

        if !has_tex {
            #[cfg(feature = "timed")] {
                crate::debug::PROFILER.render_batch(|t| t.pause());
            }
            return true;
        }

        let mut needed_tex = 0;
        let mut seen = Vec::new();
        for vertex in vertices {
            if vertex.has_texture > 0.0 {
                if !seen.contains(&vertex.texture) && !self.has_texture(vertex.texture) {
                    needed_tex += 1;
                    seen.push(vertex.texture);
                }
            }
        }

        if self.texture_index + needed_tex > MAX_TEXTURES {
            #[cfg(feature = "timed")] {
                crate::debug::PROFILER.render_batch(|t| t.pause());
            }
            return false;
        }

        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.render_batch(|t| t.pause());
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

    pub fn draw(
        &mut self,
        window: &Window,
        camera: &OrthographicCamera,
        renderer: &mut impl PrimitiveRenderer,
        shader: &mut OpenGLShader,
        back_target: &mut BackBufferTarget
    ) {
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
            shader,
            back_target
        );
        self.prepare_batch();
    }

    pub fn draw_to_target(
        &mut self,
        window: &Window,
        camera: &OrthographicCamera,
        renderer: &mut impl PrimitiveRenderer,
        shader: &mut OpenGLShader,
        post: &mut RenderTarget,
    ) {
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
            post,
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
