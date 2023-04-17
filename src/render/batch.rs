use std::rc::Rc;
use glfw::Key::N;
use mvutils::utils::{IncDec, TetrahedronOp};
use crate::render::shared::{RenderProcessor2D, Shader, Texture, TextureRegion, Window};
use std::cell::RefCell;
use std::ops::Deref;
use mvutils::init_arr;

pub(crate) const FLOAT_BYTES: u16 = 4;

pub mod batch_layout_2d {
    use crate::render::batch::FLOAT_BYTES;

    pub(crate) const POSITION_SIZE: u16 = 3;
    pub(crate) const ROTATION_SIZE: u16 = 1;
    pub(crate) const ROTATION_ORIGIN_SIZE: u16 = 2;
    pub(crate) const COLOR_SIZE: u16 = 4;
    pub(crate) const UV_SIZE: u16 = 2;
    pub(crate) const TEX_ID_SIZE: u16 = 1;
    pub(crate) const CANVAS_COORDS_SIZE: u16 = 4;
    pub(crate) const CANVAS_DATA_SIZE: u16 = 2;
    pub(crate) const USE_CAMERA_SIZE: u16 = 1;
    pub(crate) const VERTEX_SIZE_FLOATS: u16 = POSITION_SIZE + ROTATION_SIZE + ROTATION_ORIGIN_SIZE + COLOR_SIZE + UV_SIZE + TEX_ID_SIZE + CANVAS_COORDS_SIZE + CANVAS_DATA_SIZE + USE_CAMERA_SIZE;
    pub(crate) const VERTEX_SIZE_BYTES: u16 = VERTEX_SIZE_FLOATS * FLOAT_BYTES;
    pub(crate) const POSITION_OFFSET: u16 = 0;
    pub(crate) const POSITION_OFFSET_BYTES: u16 = POSITION_OFFSET * FLOAT_BYTES;
    pub(crate) const ROTATION_OFFSET: u16 = POSITION_SIZE;
    pub(crate) const ROTATION_OFFSET_BYTES: u16 = ROTATION_OFFSET * FLOAT_BYTES;
    pub(crate) const ROTATION_ORIGIN_OFFSET: u16 = ROTATION_OFFSET + ROTATION_SIZE;
    pub(crate) const ROTATION_ORIGIN_OFFSET_BYTES: u16 = ROTATION_ORIGIN_OFFSET * FLOAT_BYTES;
    pub(crate) const COLOR_OFFSET: u16 = ROTATION_ORIGIN_OFFSET + ROTATION_ORIGIN_SIZE;
    pub(crate) const COLOR_OFFSET_BYTES: u16 = COLOR_OFFSET * FLOAT_BYTES;
    pub(crate) const UV_OFFSET: u16 = COLOR_OFFSET + COLOR_SIZE;
    pub(crate) const UV_OFFSET_BYTES: u16 = UV_OFFSET * FLOAT_BYTES;
    pub(crate) const TEX_ID_OFFSET: u16 = UV_OFFSET + UV_SIZE;
    pub(crate) const TEX_ID_OFFSET_BYTES: u16 = TEX_ID_OFFSET * FLOAT_BYTES;
    pub(crate) const CANVAS_COORDS_OFFSET: u16 = TEX_ID_OFFSET + TEX_ID_SIZE;
    pub(crate) const CANVAS_COORDS_OFFSET_BYTES: u16 = CANVAS_COORDS_OFFSET * FLOAT_BYTES;
    pub(crate) const CANVAS_DATA_OFFSET: u16 = CANVAS_COORDS_OFFSET + CANVAS_COORDS_SIZE;
    pub(crate) const CANVAS_DATA_OFFSET_BYTES: u16 = CANVAS_DATA_OFFSET * FLOAT_BYTES;
    pub(crate) const USE_CAMERA_OFFSET: u16 = CANVAS_DATA_OFFSET + CANVAS_DATA_OFFSET_BYTES;
    pub(crate) const USE_CAMERA_OFFSET_BYTES: u16 = USE_CAMERA_OFFSET * FLOAT_BYTES;
}

pub(crate) trait BatchGen {
    fn get_render_mode(&self) -> u8;
    fn gen_indices(&self, amt: u16, offset: u32, indices: &mut Vec<u32>);
    fn is_stripped(&self) -> bool;
}

struct RegularBatch;
struct StrippedBatch;

impl BatchGen for RegularBatch {
    fn get_render_mode(&self) -> u8 {
        gl::TRIANGLES as u8
    }

    fn gen_indices(&self, amt: u16, offset: u32, indices: &mut Vec<u32>) {
        if amt == 4 {
            indices[(offset * 6 + 0) as usize] = 0 + offset * 4;
            indices[(offset * 6 + 1) as usize] = 1 + offset * 4;
            indices[(offset * 6 + 2) as usize] = 2 + offset * 4;
            indices[(offset * 6 + 3) as usize] = 0 + offset * 4;
            indices[(offset * 6 + 4) as usize] = 2 + offset * 4;
            indices[(offset * 6 + 5) as usize] = 3 + offset * 4;
        } else {
            indices.insert((offset * 6 + 0) as usize, 0 + offset * 4);
            indices.insert((offset * 6 + 1) as usize, 1 + offset * 4);
            indices.insert((offset * 6 + 2) as usize, 2 + offset * 4);
        }
    }

    fn is_stripped(&self) -> bool {
        false
    }
}

impl BatchGen for StrippedBatch {
    fn get_render_mode(&self) -> u8 {
        gl::TRIANGLE_STRIP as u8
    }

    fn gen_indices(&self, amt: u16, offset: u32, indices: &mut Vec<u32>) {
        if amt == 4 {
            indices[(offset * 6 + 0) as usize] = 0 + offset * 4;
            indices[(offset * 6 + 1) as usize] = 1 + offset * 4;
            indices[(offset * 6 + 2) as usize] = 2 + offset * 4;
            indices[(offset * 6 + 3) as usize] = 0 + offset * 4;
            indices[(offset * 6 + 4) as usize] = 2 + offset * 4;
            indices[(offset * 6 + 5) as usize] = 3 + offset * 4;
        } else {
            indices[(offset * 6 + 0) as usize] = 0 + offset * 4;
            indices[(offset * 6 + 1) as usize] = 1 + offset * 4;
            indices[(offset * 6 + 2) as usize] = 2 + offset * 4;
        }
    }

    fn is_stripped(&self) -> bool {
        true
    }
}

struct Batch2D {
    generator: Box<dyn BatchGen>,
    data: Vec<f32>,
    indices: Vec<u32>,
    textures: [Option<Rc<RefCell<Texture>>>; 17],
    tex_ids: [u32; 17],
    shader: Rc<RefCell<Shader>>,
    vbo: u32,
    ibo: u32,
    size: u32,
    vert_count: u32,
    obj_count: u32,
    next_tex: u32,
    full: bool,
    full_tex: bool
}

impl Batch2D {
    pub(crate) fn new<T: BatchGen + 'static>(size: u32, shader: Rc<RefCell<Shader>>, generator: T) -> Self {
        Batch2D {
            generator: Box::new(generator),
            data: Vec::with_capacity(size as usize * batch_layout_2d::VERTEX_SIZE_FLOATS as usize),
            indices: Vec::with_capacity(size as usize * 6),
            textures: [0; 17].map(|n| None),
            tex_ids: [0; 17],
            shader,
            vbo: 0,
            ibo: 0,
            size,
            vert_count: 0,
            obj_count: 0,
            next_tex: 0,
            full: false,
            full_tex: false
        }
    }

    pub(crate) fn clear(&mut self) {
        self.vert_count = 0;
        self.obj_count = 0;
        self.next_tex = 0;

        self.full = false;
        self.full_tex = false;
    }

    pub(crate) fn force_clear(&mut self) {
        self.data.clear();
        self.indices.clear();
        self.textures.fill(None);
        self.tex_ids.fill(0);

        self.clear();
    }

    pub(crate) fn is_full(&self, amount: u32) -> bool {
        self.data.capacity() < amount as usize
    }

    pub(crate) fn is_full_tex(&self) -> bool {
        self.full_tex
    }

    fn add_vertex(&mut self, vertex: &Vertex2D) {
        for i in 0..batch_layout_2d::VERTEX_SIZE_FLOATS {
            self.data.insert(i as usize + (self.vert_count * batch_layout_2d::VERTEX_SIZE_FLOATS as u32) as usize, vertex.data[i as usize]);
        }
        self.vert_count += 1;
    }

    pub(crate) fn add_vertices(&mut self, vertices: &VertexGroup<Vertex2D>) {
        if self.is_full(vertices.len as u32) {
            return;
        }

        self.generator.gen_indices(vertices.len as u16, self.obj_count, &mut self.indices);

        for i in 0..vertices.len {
            self.add_vertex(vertices.get(i));
            if self.vert_count > self.size {
                self.full = true;
                return;
            }
        }
        if vertices.len < 4 {
            self.add_vertex(vertices.get(0));
            if self.vert_count > self.size {
                self.full = true;
                return;
            }
        }

        self.obj_count += 1;
    }

    pub(crate) fn add_texture(&mut self, texture: Rc<RefCell<Texture>>) -> u32 {
        if self.full_tex {
            return 0;
        }

        for i in 0..17 {
            if let Some(tex) = &self.textures[i] {
                if tex.borrow().get_id() == texture.borrow().get_id() {
                    return i as u32 + 1;
                }
            }
        }

        self.textures[self.next_tex as usize] = Some(texture);
        self.tex_ids[self.next_tex as usize] = self.next_tex + 1;
        self.next_tex += 1;

        if self.next_tex >= 17 {
            self.full_tex = true;
        }

        return self.next_tex;
    }

    pub(crate) fn render(&mut self, processor: &impl RenderProcessor2D) {
        if self.vbo == 0 {
            self.vbo = processor.gen_buffer_id();
        }
        if self.ibo == 0 {
            self.ibo = processor.gen_buffer_id();
        }
        processor.process_data(&mut self.textures, &self.tex_ids, &self.indices, &self.data, self.vbo, self.ibo, self.shader.borrow().deref(), processor.adapt_render_mode(self.generator.get_render_mode()));
        self.force_clear();
    }

    pub(crate) fn set_shader(&mut self, shader: Rc<RefCell<Shader>>) {
        self.shader = shader;
    }

    pub(crate) fn is_stripped(&self) -> bool {
        self.generator.is_stripped()
    }
}

//data storage

pub(crate) struct Vertex2D {
    data: [f32; batch_layout_2d::VERTEX_SIZE_FLOATS as usize]
}

impl Vertex2D {
    pub(crate) fn new() -> Self {
        Vertex2D { data: [0.0; batch_layout_2d::VERTEX_SIZE_FLOATS as usize] }
    }

    pub(crate) fn set(&mut self, data: [f32; batch_layout_2d::VERTEX_SIZE_FLOATS as usize]) {
        self.data = data;
    }
}

pub(crate) struct VertexGroup<T> {
    vertices: [T; 4],
    len: u8
}

impl VertexGroup<Vertex2D> {
    pub(crate) fn new() -> Self {
        VertexGroup {
            vertices: init_arr!(4, Vertex2D::new()),
            len: 0
        }
    }

    pub(crate) fn get(&self, index: u8) -> &Vertex2D {
        assert!(index <= 4);
        &self.vertices[index as usize]
    }

    pub(crate) fn get_mut(&mut self, index: u8) -> &mut Vertex2D {
        assert!(index <= 4);
        &mut self.vertices[index as usize]
    }

    pub(crate) fn set_len(&mut self, len: u8) {
        self.len = len;
    }

    pub(crate) fn len(&self) -> u8 {
        self.len
    }
}

//controllers

pub(crate) struct BatchController2D {
    batches: Vec<Batch2D>,
    batch_limit: u32,
    shader: Rc<RefCell<Shader>>,
    default_shader: Rc<RefCell<Shader>>,
    current: u32
}

impl BatchController2D {
    pub(crate) fn new(shader: Rc<RefCell<Shader>>, batch_limit: u32) -> Self {
        assert!(batch_limit >= 14);
        shader.borrow_mut().make();
        shader.borrow_mut().bind();
        let mut batch = BatchController2D {
            batches: Vec::new(),
            batch_limit,
            default_shader: shader.clone(),
            shader,
            current: 0
        };
        batch.start();
        batch
    }

    pub(crate) fn start(&mut self) {
        self.batches.push(Batch2D::new(self.batch_limit, self.shader.clone(), RegularBatch));
    }

    fn next_batch(&mut self, stripped: bool) {
        self.current += 1;
        if let Some(batch) = self.batches.get(self.current as usize) {
            if batch.is_stripped() != stripped {
                self.batches.insert(self.current as usize, self.gen_batch(stripped))
            }
        } else {
            self.batches.push(self.gen_batch(stripped));
        }

    }

    fn gen_batch(&self, stripped: bool) -> Batch2D {
        stripped.yn(Batch2D::new(self.batch_limit, self.shader.clone(), StrippedBatch),
                    Batch2D::new(self.batch_limit, self.shader.clone(), RegularBatch))
    }

    pub(crate) fn add_vertices(&mut self, vertices: &VertexGroup<Vertex2D>) {
        if self.batches[self.current as usize].is_stripped() {
            self.next_batch(false);
        }
        if self.batches[self.current as usize ].is_full(vertices.len as u32 * batch_layout_2d::VERTEX_SIZE_FLOATS as u32) {
            self.next_batch(false);
        }

        self.batches[self.current as usize].add_vertices(vertices);
    }

    pub(crate) fn add_vertices_stripped(&mut self, vertices: &VertexGroup<Vertex2D>) {
        if !self.batches[self.current as usize].is_stripped() {
            self.next_batch(true);
        }
        if self.batches[self.current as usize].is_full(vertices.len as u32 * batch_layout_2d::VERTEX_SIZE_FLOATS as u32) {
            self.next_batch(true);
        }

        self.batches[self.current as usize].add_vertices(vertices);
    }

    pub(crate) fn add_texture(&mut self, texture: Rc<RefCell<Texture>>, vert_count: u32) -> u32 {
        if self.batches[self.current as usize].is_stripped() {
            self.next_batch(false);
        }
        if self.batches[self.current as usize].is_full(vert_count * batch_layout_2d::VERTEX_SIZE_FLOATS as u32) {
            self.next_batch(false);
        }

        self.batches[self.current as usize].add_texture(texture)
    }

    pub(crate) fn add_texture_stripped(&mut self, texture: Rc<RefCell<Texture>>, vert_count: u32) -> u32 {
        if !self.batches[self.current as usize].is_stripped() {
            self.next_batch(true);
        }
        if self.batches[self.current as usize].is_full(vert_count * batch_layout_2d::VERTEX_SIZE_FLOATS as u32) {
            self.next_batch(true);
        }

        self.batches[self.current as usize].add_texture(texture)
    }

    pub(crate) fn render(&mut self, processor: &impl RenderProcessor2D) {
        for i in 0..self.current + 1 {
            self.batches[i as usize].render(processor);
        }
        self.current = 0;
    }

    pub(crate) fn set_shader(&mut self, shader: Rc<RefCell<Shader>>) {
        self.shader = shader;
        self.shader.borrow_mut().make();
        for batch in self.batches.iter_mut() {
            batch.set_shader(self.shader.clone());
        }
    }

    pub(crate) fn reset_shader(&mut self) {
        self.set_shader(self.default_shader.clone());
    }
}