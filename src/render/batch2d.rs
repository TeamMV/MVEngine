use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

use mvutils::init_arr;
use mvutils::utils::{TetrahedronOp};
use crate::render::color::{Color, RGB};

use crate::render::shared::{RenderProcessor2D, Shader, Texture};

pub(crate) const FLOAT_BYTES: u16 = 4;

pub mod batch_layout_2d {
    use crate::render::batch2d::FLOAT_BYTES;

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
    pub(crate) const USE_CAMERA_OFFSET: u16 = CANVAS_DATA_OFFSET + CANVAS_DATA_SIZE;
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
            indices.insert((offset * 6 + 0) as usize, 0 + offset * 4);
            indices.insert((offset * 6 + 1) as usize, 1 + offset * 4);
            indices.insert((offset * 6 + 2) as usize, 2 + offset * 4);
            indices.insert((offset * 6 + 3) as usize, 0 + offset * 4);
            indices.insert((offset * 6 + 4) as usize, 2 + offset * 4);
            indices.insert((offset * 6 + 5) as usize, 3 + offset * 4);
        } else {
            indices.insert((offset * 6 + 0) as usize, 0 + offset * 4);
            indices.insert((offset * 6 + 1) as usize, 1 + offset * 4);
            indices.insert((offset * 6 + 2) as usize, 2 + offset * 4);
            indices.insert((offset * 6 + 3) as usize, 0);
            indices.insert((offset * 6 + 4) as usize, 0);
            indices.insert((offset * 6 + 5) as usize, 0);
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
            indices.insert((offset * 6 + 0) as usize, 0 + offset * 4);
            indices.insert((offset * 6 + 1) as usize, 1 + offset * 4);
            indices.insert((offset * 6 + 2) as usize, 2 + offset * 4);
            indices.insert((offset * 6 + 3) as usize, 0 + offset * 4);
            indices.insert((offset * 6 + 4) as usize, 2 + offset * 4);
            indices.insert((offset * 6 + 5) as usize, 3 + offset * 4);
        } else {
            indices.insert((offset * 6 + 0) as usize, 0 + offset * 4);
            indices.insert((offset * 6 + 1) as usize, 1 + offset * 4);
            indices.insert((offset * 6 + 2) as usize, 2 + offset * 4);
            indices.insert((offset * 6 + 3) as usize, 0);
            indices.insert((offset * 6 + 4) as usize, 0);
            indices.insert((offset * 6 + 5) as usize, 0);
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
    tex_ids: [u32; 16],
    size: u32,
    vert_count: u32,
    obj_count: u32,
    next_tex: u32,
    full: bool,
    full_tex: bool,
}

impl Batch2D {
    fn new<T: BatchGen + 'static>(size: u32, generator: T) -> Self {
        Batch2D {
            generator: Box::new(generator),
            data: Vec::with_capacity(size as usize * batch_layout_2d::VERTEX_SIZE_FLOATS as usize),
            indices: Vec::with_capacity(size as usize * 6),
            textures: [0; 17].map(|_| None),
            tex_ids: [0; 16],
            size,
            vert_count: 0,
            obj_count: 0,
            next_tex: 0,
            full: false,
            full_tex: false,
        }
    }

    fn clear(&mut self) {
        self.vert_count = 0;
        self.obj_count = 0;
        self.next_tex = 0;

        self.full = false;
        self.full_tex = false;
    }

    fn force_clear(&mut self) {
        self.data.clear();
        self.indices.clear();
        self.textures.fill(None);
        self.tex_ids.fill(0);

        self.clear();
    }

    fn is_full(&self, amount: u32) -> bool {
        self.data.capacity() < amount as usize
    }

    fn is_full_tex(&self) -> bool {
        self.full_tex
    }

    fn add_vertex(&mut self, vertex: &Vertex2D) {
        for i in 0..batch_layout_2d::VERTEX_SIZE_FLOATS {
            self.data.insert(i as usize + (self.vert_count * batch_layout_2d::VERTEX_SIZE_FLOATS as u32) as usize, vertex.data[i as usize]);
        }
        self.vert_count += 1;
    }

    fn add_vertices(&mut self, vertices: &VertexGroup<Vertex2D>) {
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

    fn add_texture(&mut self, texture: Rc<RefCell<Texture>>) -> u32 {
        if self.full_tex {
            return 0;
        }

        for i in 0..16 {
            if let Some(tex) = &self.textures[i] {
                if tex.borrow().get_id() == texture.borrow().get_id() {
                    return i as u32 + 1;
                }
            }
        }

        self.textures[self.next_tex as usize] = Some(texture);
        self.tex_ids[self.next_tex as usize] = self.next_tex;
        self.next_tex += 1;

        if self.next_tex >= 17 {
            self.full_tex = true;
        }

        self.next_tex
    }

    fn render(&mut self, processor: &impl RenderProcessor2D, shader: &mut Shader) {
        processor.process_data(&mut self.textures, &self.tex_ids, &self.indices, &self.data, shader,  self.generator.get_render_mode());
        self.force_clear();
    }

    fn is_stripped(&self) -> bool {
        self.generator.is_stripped()
    }
}

//data storage

pub(crate) struct Vertex2D {
    data: [f32; batch_layout_2d::VERTEX_SIZE_FLOATS as usize],
}

#[allow(clippy::too_many_arguments)]
impl Vertex2D {
    pub(crate) fn new() -> Self {
        Vertex2D { data: [0.0; batch_layout_2d::VERTEX_SIZE_FLOATS as usize] }
    }

    pub(crate) fn set(&mut self, data: [f32; batch_layout_2d::VERTEX_SIZE_FLOATS as usize]) {
        self.data = data;
    }

    pub(crate) fn set_data(&mut self, x: f32, y: f32, z: f32, rot: f32, rx: f32, ry: f32, col: Color<RGB, f32>, canvas: [f32; 6], cam: bool) {
        self.set([x, y, z, rot, rx, ry, col.r(), col.g(), col.b(), col.a(), 0.0, 0.0, 0.0, canvas[0], canvas[1], canvas[2], canvas[3], canvas[4], canvas[5], cam.yn(1.0, 0.0)]);
    }

    pub(crate) fn set_texture_data(&mut self, x: f32, y: f32, z: f32, rot: f32, rx: f32, ry: f32, col: Color<RGB, f32>, ux: f32, uy: f32, tex: u32, canvas: [f32; 6], cam: bool) {
        self.set([x, y, z, rot, rx, ry, col.r(), col.g(), col.b(), col.a(), ux, uy, tex as f32, canvas[0], canvas[1], canvas[2], canvas[3], canvas[4], canvas[5], cam.yn(1.0, 0.0)]);
    }

    pub(crate) fn set_norot_texture_data(&mut self, x: f32, y: f32, z: f32, col: Color<RGB, f32>, ux: f32, uy: f32, tex: u32, canvas: [f32; 6], cam: bool) {
        self.set([x, y, z, 0.0, 0.0, 0.0, col.r(), col.g(), col.b(), col.a(), ux, uy, tex as f32, canvas[0], canvas[1], canvas[2], canvas[3], canvas[4], canvas[5], cam.yn(1.0, 0.0)]);
    }
}

pub(crate) struct VertexGroup<T> {
    vertices: [T; 4],
    len: u8,
}

impl VertexGroup<Vertex2D> {
    pub(crate) fn new() -> Self {
        VertexGroup {
            vertices: init_arr!(4, Vertex2D::new()),
            len: 0,
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
    current: u32,
}

impl BatchController2D {
    pub(crate) fn new(shader: Rc<RefCell<Shader>>, batch_limit: u32) -> Self {
        assert!(batch_limit >= 14);
        let mut batch = BatchController2D {
            batches: Vec::new(),
            batch_limit,
            default_shader: shader.clone(),
            shader,
            current: 0,
        };
        batch.start();
        batch
    }

    fn start(&mut self) {
        self.batches.push(Batch2D::new(self.batch_limit, RegularBatch));
    }

    fn next_batch(&mut self, stripped: bool) {
        self.current += 1;
        if let Some(batch) = self.batches.get(self.current as usize) {
            if batch.is_stripped() != stripped {
                self.batches[self.current as usize] = self.gen_batch(stripped);
            }
        } else {
            self.batches.push(self.gen_batch(stripped));
        }
    }

    fn gen_batch(&self, stripped: bool) -> Batch2D {
        stripped.yn(Batch2D::new(self.batch_limit, StrippedBatch),
                    Batch2D::new(self.batch_limit, RegularBatch))
    }

    pub(crate) fn add_vertices(&mut self, vertices: &VertexGroup<Vertex2D>) {
        if self.batches[self.current as usize].is_stripped() {
            self.next_batch(false);
        }
        if self.batches[self.current as usize].is_full(vertices.len as u32 * batch_layout_2d::VERTEX_SIZE_FLOATS as u32) {
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
        texture.borrow_mut().make();

        if self.batches[self.current as usize].is_stripped() {
            self.next_batch(false);
        }
        if self.batches[self.current as usize].is_full(vert_count * batch_layout_2d::VERTEX_SIZE_FLOATS as u32) {
            self.next_batch(false);
        }

        self.batches[self.current as usize].add_texture(texture)
    }

    pub(crate) fn add_texture_stripped(&mut self, texture: Rc<RefCell<Texture>>, vert_count: u32) -> u32 {
        texture.borrow_mut().make();

        if !self.batches[self.current as usize].is_stripped() {
            self.next_batch(true);
        }
        if self.batches[self.current as usize].is_full(vert_count * batch_layout_2d::VERTEX_SIZE_FLOATS as u32) {
            self.next_batch(true);
        }

        self.batches[self.current as usize].add_texture(texture)
    }

    pub(crate) fn render(&mut self, processor: &impl RenderProcessor2D) {
        self.shader.borrow_mut().bind();
        for i in 0..self.current + 1 {
            self.batches[i as usize].render(processor, self.shader.borrow_mut().deref_mut());
        }
        self.current = 0;
    }

    pub(crate) fn set_shader(&mut self, shader: Rc<RefCell<Shader>>) {
        self.shader = shader;
    }

    pub(crate) fn reset_shader(&mut self) {
        self.set_shader(self.default_shader.clone());
    }
}