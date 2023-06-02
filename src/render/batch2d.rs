use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::Arc;
use itertools::Itertools;

use mvutils::init_arr;
use mvutils::utils::{TetrahedronOp};
use crate::render::color::{Color, RGB};

use crate::render::common::{Shader, Texture};
use crate::render::consts::{INDEX_LIMIT, MAX_TEXTURES, TEXTURE_LIMIT, VERT_LIMIT, VERT_LIMIT_2D_BYTES, VERT_LIMIT_2D_FLOATS};
use crate::render::init::PipelineBuilder;
use crate::render::render::RenderPass2D;

pub(crate) const FLOAT_BYTES: u16 = 4;

pub mod batch_layout_2d {
    const FLOAT_BYTES: u16 = 4;

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

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum BatchType {
    Regular,
    Stripped
}

trait BatchGen {
    fn get_render_mode(&self) -> u8;
    fn gen_indices(&self, amt: u16, offset: u32, indices: &mut Vec<u32>);
    fn batch_type(&self) -> BatchType;
}

struct RegularBatch;

struct StrippedBatch;

impl BatchGen for RegularBatch {
    fn get_render_mode(&self) -> u8 {
        PipelineBuilder::RENDER_MODE_TRIANGLES
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

    fn batch_type(&self) -> BatchType {
        BatchType::Regular
    }
}

impl BatchGen for StrippedBatch {
    fn get_render_mode(&self) -> u8 {
        PipelineBuilder::RENDER_MODE_TRIANGLE_STRIP
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

    fn batch_type(&self) -> BatchType {
        BatchType::Stripped
    }
}

struct Batch2D {
    generator: Box<dyn BatchGen>,
    data: Vec<f32>,
    indices: Vec<u32>,
    textures: [Option<Arc<Texture>>; TEXTURE_LIMIT as usize],
    tex_ids: [u32; TEXTURE_LIMIT as usize],
    vert_count: u32,
    obj_count: u32,
    next_tex: u32,
    full: bool,
    full_tex: bool,
}

impl Batch2D {
    fn new<T: BatchGen + 'static>(generator: T) -> Self {
        Batch2D {
            generator: Box::new(generator),
            data: Vec::with_capacity(VERT_LIMIT_2D_FLOATS as usize),
            indices: Vec::with_capacity(INDEX_LIMIT as usize),
            textures: [0; TEXTURE_LIMIT as usize].map(|_| None),
            tex_ids: [0; TEXTURE_LIMIT as usize],
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

    fn is_empty(&self) -> bool {
        self.vert_count == 0
    }

    fn is_full_tex(&self) -> bool {
        self.full_tex
    }

    fn is_full_tex_for(&self, amount: u32) -> bool {
        self.next_tex + amount > unsafe { *MAX_TEXTURES } as u32
    }

    fn can_hold(&self, vertices: u32, textures: u32) -> bool {
        !(self.is_full(vertices) || self.is_full_tex_for(textures))
    }

    fn add_vertex(&mut self, vertex: &Vertex2D) {
        for i in 0..batch_layout_2d::VERTEX_SIZE_FLOATS {
            self.data.insert(i as usize + (self.vert_count * batch_layout_2d::VERTEX_SIZE_FLOATS as u32) as usize, vertex.data[i as usize]);
        }
        self.vert_count += 1;
    }

    fn end_fan(&mut self) {
        self.generator.gen_indices(self.vert_count as u16, 0, &mut self.indices);
    }

    fn add_vertices(&mut self, vertices: &VertexGroup<Vertex2D>) {
        if self.is_full(vertices.len as u32) {
            return;
        }

        self.generator.gen_indices(vertices.len as u16, self.obj_count, &mut self.indices);

        for i in 0..vertices.len {
            self.add_vertex(vertices.get(i));
            if self.vert_count > VERT_LIMIT as u32 {
                self.full = true;
                return;
            }
        }
        if vertices.len < 4 {
            self.add_vertex(vertices.get(0));
            if self.vert_count > VERT_LIMIT as u32 {
                self.full = true;
                return;
            }
        }

        self.obj_count += 1;
    }

    fn add_texture(&mut self, texture: Arc<Texture>) -> u32 {
        if self.full_tex {
            return 0;
        }

        for i in 0..unsafe { *MAX_TEXTURES } {
            if let Some(tex) = &self.textures[i] {
                if tex == &texture {
                    return i as u32 + 1;
                }
            }
        }

        self.textures[self.next_tex as usize] = Some(texture);
        self.tex_ids[self.next_tex as usize] = self.next_tex;
        self.next_tex += 1;

        if self.next_tex > unsafe { *MAX_TEXTURES } as u32 {
            self.full_tex = true;
        }

        self.next_tex
    }

    fn render(&mut self, render_pass: &mut RenderPass2D) {
        render_pass.render(self.indices.as_slice(), self.data.as_slice(), &self.textures, (self.generator.get_render_mode() == PipelineBuilder::RENDER_MODE_TRIANGLE_STRIP).yn(true, false));
        self.force_clear();

    }

    fn batch_type(&self) -> BatchType {
        self.generator.batch_type()
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

    fn z(&self, z: f32) {
        unsafe {
            let ptr = self as *const Vertex2D;
            let v = ptr.cast_mut().as_mut().unwrap();
            v.data[2] = z;
        }
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

    fn z(&self, z: f32) {
        self.vertices[0].z(z);
        self.vertices[1].z(z);
        self.vertices[2].z(z);
        self.vertices[3].z(z);
    }
}

//controllers

pub(crate) struct BatchController2D {
    batches: Vec<Batch2D>,
    current: u32,
    previous_regular: i32,
    z: f32
}

impl BatchController2D {
    const Z_SHIFT: f32 = 0.01;
    const Z_BASE: f32 = 1999.0;

    pub(crate) fn new() -> Self {
        let mut batch = BatchController2D {
            batches: Vec::new(),
            current: 0,
            previous_regular: -1,
            z: Self::Z_BASE,
        };
        batch.start();
        batch
    }

    fn start(&mut self) {
        self.batches.push(Batch2D::new(RegularBatch));
    }

    fn ensure_batch(&mut self, batch_type: BatchType, vertices: u32, textures: u32) {
        match batch_type {
            BatchType::Regular => {
                if self.batches[self.current as usize].batch_type() != batch_type {
                    if self.previous_regular >= 0 {
                        if self.batches[self.previous_regular as usize].can_hold(vertices, textures) {
                            self.inc_z();
                            return;
                        }
                    }
                    self.advance(batch_type);
                    self.previous_regular = self.current as i32;
                }
                else {
                    if self.batches[self.current as usize].can_hold(vertices, textures) {
                        self.inc_z();
                        //NEW!
                        self.previous_regular = self.current as i32;
                        return;
                    }
                    self.advance(batch_type);
                    self.previous_regular = self.current as i32;
                }
            }
            BatchType::Stripped => {
                if self.batches[self.current as usize].batch_type() == batch_type {
                    if self.batches[self.current as usize].is_empty() {
                        return;
                    }
                }
                self.advance(batch_type);
            }
        }
    }

    fn inc_z(&mut self) {
        self.z -= Self::Z_SHIFT;
        if self.z <= 0.1 {
            self.z = 0.1;
        }
    }

    fn advance(&mut self, batch_type: BatchType) {
        self.current += 1;
        if self.batches.len() > self.current as usize {
            if self.batches[self.current as usize].batch_type() != batch_type {
                self.batches[self.current as usize] = self.gen_batch(batch_type);
            }
        } else {
            self.batches.push(self.gen_batch(batch_type));
        }
    }

    fn gen_batch(&self, batch_type: BatchType) -> Batch2D {
        match batch_type {
            BatchType::Regular => Batch2D::new(RegularBatch),
            BatchType::Stripped => Batch2D::new(StrippedBatch),
        }
    }

    pub(crate) fn add_vertices(&mut self, vertices: &VertexGroup<Vertex2D>) {
        vertices.z(self.z);
        self.ensure_batch(BatchType::Regular, vertices.len() as u32, 0);
        self.batches[self.previous_regular as usize].add_vertices(vertices);
    }

    pub(crate) fn start_stripped(&mut self) {
        self.ensure_batch(BatchType::Stripped, 0, 0);
    }

    pub(crate) fn add_vertices_stripped(&mut self, vertices: &VertexGroup<Vertex2D>) {
        vertices.z(self.z);
        if self.batches[self.current as usize].batch_type() == BatchType::Stripped {
            self.batches[self.current as usize].add_vertices(vertices);
        }
    }

    pub(crate) fn require(&mut self, vertices: u32, textures: u32) {
        self.ensure_batch(BatchType::Regular, vertices, textures);
    }

    pub(crate) fn add_texture(&mut self, texture: Arc<Texture>, vert_count: u32) -> u32 {
        self.ensure_batch(BatchType::Regular, vert_count, 1);
        self.batches[self.previous_regular as usize].add_texture(texture)
    }

    pub(crate) fn add_texture_stripped(&mut self, texture: Arc<Texture>) -> u32 {
        if self.batches[self.current as usize].batch_type() == BatchType::Stripped {
            self.batches[self.current as usize].add_texture(texture)
        }
        else {
            0
        }
    }

    pub(crate) fn render(&mut self, processor: &mut RenderPass2D) {
        for i in 0..=self.current {
            self.batches[i as usize].render(processor);
        }
        self.current = 0;
        self.previous_regular = -1;
        self.z = Self::Z_BASE;
    }
}