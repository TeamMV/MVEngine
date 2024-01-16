use std::sync::Arc;

use crate::render::batch2d::{BatchGen, BatchType};
use glam::Mat4;

use crate::render::common::Texture;
use crate::render::common3d::{Model, ModelArray};
use crate::render::consts::{MAX_TEXTURES, TEXTURE_LIMIT, VERTEX_3D_SIZE_FLOATS};
use crate::render::init::PipelineBuilder;
use crate::render::render3d::RenderPass3D;

struct RegularBatch;

struct StrippedBatch;

impl BatchGen for RegularBatch {
    fn get_render_mode(&self) -> u8 {
        PipelineBuilder::RENDER_MODE_TRIANGLES
    }

    fn batch_type(&self) -> BatchType {
        BatchType::Regular
    }
}

impl BatchGen for StrippedBatch {
    fn get_render_mode(&self) -> u8 {
        PipelineBuilder::RENDER_MODE_TRIANGLE_STRIP
    }

    fn batch_type(&self) -> BatchType {
        BatchType::Stripped
    }
}

struct Batch3D {
    generator: Box<dyn BatchGen>,
    data: Vec<f32>,
    indices: Vec<u32>,
    textures: [Option<Arc<Texture>>; TEXTURE_LIMIT],
    transformations: Vec<Mat4>,
    tex_ids: [u32; TEXTURE_LIMIT],
    size: u32,
    vert_count: u32,
    obj_count: u32,
    next_tex: u32,
    full: bool,
    full_tex: bool,
}

impl Batch3D {
    fn new<T: BatchGen + 'static>(size: u32, generator: T) -> Self {
        Batch3D {
            generator: Box::new(generator),
            data: Vec::with_capacity(size as usize),
            indices: Vec::with_capacity(size as usize * 6),
            textures: [0; TEXTURE_LIMIT].map(|_| None),
            transformations: vec![],
            tex_ids: [0; TEXTURE_LIMIT],
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

    fn is_empty(&self) -> bool {
        self.vert_count == 0
    }

    fn is_full_tex(&self) -> bool {
        self.full_tex
    }

    fn is_full_tex_for(&self, amount: u32) -> bool {
        self.next_tex + amount < *MAX_TEXTURES as u32
    }

    fn can_hold(&self, vertices: u32, textures: u32) -> bool {
        !(self.is_full(vertices) || self.is_full_tex_for(textures))
    }

    fn add_texture(&mut self, texture: Arc<Texture>) -> u32 {
        if self.full_tex {
            return 0;
        }

        for i in 0..*MAX_TEXTURES {
            if let Some(tex) = &self.textures[i] {
                if tex.get_id() == texture.get_id() {
                    return i as u32 + 1;
                }
            }
        }

        self.textures[self.next_tex as usize] = Some(texture);
        self.tex_ids[self.next_tex as usize] = self.next_tex;
        self.next_tex += 1;

        if self.next_tex > *MAX_TEXTURES as u32 {
            self.full_tex = true;
        }

        self.next_tex
    }

    fn push_model(&mut self, model: Model, canvas: [f32; 6], model_matrix: Mat4) {}

    fn batch_type(&self) -> BatchType {
        self.generator.batch_type()
    }
}

enum RenderType3D {
    Batch(Batch3D),
    Model(Model),
    ModelArray(ModelArray),
}

impl RenderType3D {
    fn is_model(&self) -> bool {
        matches!(self, RenderType3D::Model(_))
    }

    fn is_batch(&self) -> bool {
        matches!(self, RenderType3D::Batch(_))
    }

    fn get_model(&mut self) -> &mut Model {
        match self {
            RenderType3D::Model(model) => model,
            _ => unreachable!(),
        }
    }

    fn get_batch(&mut self) -> &mut Batch3D {
        match self {
            RenderType3D::Batch(batch) => batch,
            _ => unreachable!(),
        }
    }

    fn render(&self, processor: &impl RenderPass3D) {
        todo!()
    }

    fn batch_type(&self) -> BatchType {
        match self {
            RenderType3D::Batch(batch) => batch.batch_type(),
            _ => unreachable!(),
        }
    }

    fn can_hold(&self, vertices: u32, textures: u32) -> bool {
        match self {
            RenderType3D::Batch(batch) => batch.can_hold(vertices, textures),
            _ => unreachable!(),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            RenderType3D::Batch(batch) => batch.is_empty(),
            _ => unreachable!(),
        }
    }
}

//data

pub(crate) union Vertex3D {
    data: [f32; VERTEX_3D_SIZE_FLOATS],
}

impl Vertex3D {
    pub fn new() -> Self {
        Vertex3D {
            data: [0.0; VERTEX_3D_SIZE_FLOATS],
        }
    }

    pub(crate) fn set(&mut self, data: [f32; VERTEX_3D_SIZE_FLOATS]) {
        self.data = data;
    }

    pub(crate) fn set_data(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        nx: f32,
        ny: f32,
        nz: f32,
        ux: f32,
        uy: f32,
        material_id: u16,
        canvas: [f32; 6],
        model_matrix: u32,
    ) {
        self.set([
            x,
            y,
            z,
            nx,
            ny,
            nz,
            ux,
            uy,
            material_id as f32,
            model_matrix as f32,
        ]);
    }
}

//controller

pub(crate) struct BatchController3D {
    batches: Vec<RenderType3D>,
    batch_limit: u32,
    model_limit: u32,
    current: u32,
    previous: i32,
}

impl BatchController3D {
    pub(crate) fn new(batch_limit: u32) -> Self {
        assert!(batch_limit > 14);
        let mut batch = BatchController3D {
            batches: Vec::new(),
            batch_limit,
            model_limit: (batch_limit * 3) / 4,
            current: 0,
            previous: -1,
        };
        batch.start();
        batch
    }

    fn start(&mut self) {
        self.batches.push(self.gen_batch(BatchType::Regular));
    }

    fn gen_batch(&self, batch_type: BatchType) -> RenderType3D {
        match batch_type {
            BatchType::Regular => RenderType3D::Batch(Batch3D::new(self.batch_limit, RegularBatch)),
            BatchType::Stripped => {
                RenderType3D::Batch(Batch3D::new(self.batch_limit, StrippedBatch))
            }
        }
    }

    fn ensure_batch(&mut self, batch_type: BatchType, vertices: u32, textures: u32) {
        if self.batches[self.current as usize].is_model() {
            if batch_type == BatchType::Stripped {
                self.advance(batch_type);
                return;
            } else {
                if self.previous >= 0
                    && self.batches[self.previous as usize].can_hold(vertices, textures)
                {
                    return;
                }
                self.current += 1;
                self.previous = self.current as i32;
                self.advance(batch_type);
                return;
            }
        }
        match batch_type {
            BatchType::Regular => {
                if self.batches[self.current as usize].batch_type() != batch_type {
                    if self.previous >= 0
                        && self.batches[self.previous as usize].can_hold(vertices, textures)
                    {
                        return;
                    }
                    self.advance(batch_type);
                    self.previous = self.current as i32;
                } else {
                    if self.batches[self.current as usize].can_hold(vertices, textures) {
                        return;
                    }
                    self.advance(batch_type);
                    self.previous = self.current as i32;
                }
            }
            BatchType::Stripped => {
                if self.batches[self.current as usize].batch_type() == batch_type
                    && self.batches[self.current as usize].is_empty()
                {
                    return;
                }
                self.advance(batch_type);
            }
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

    fn push_model(&mut self, model: Model, canvas: [f32; 6], model_matrix: Mat4) {
        let model = RenderType3D::Model(model);
        self.current += 1;
        if self.batches.len() > self.current as usize {
            self.batches[self.current as usize] = model;
        } else {
            self.batches.push(model);
        }
    }

    pub(crate) fn add_model(&mut self, model: Model, canvas: [f32; 6], model_matrix: Mat4) {
        //if model.borrow().is_simple_geometry() && model.borrow().vertex_count() <= self.model_limit {
        //    self.ensure_batch(BatchType3D::Regular, model.borrow().vertex_count(), model.borrow().texture_count(TextureType::Geometry));
        //    self.batches[self.current as usize].get_batch().push_model(model, canvas, model_matrix);
        //}
        //else {
        self.push_model(model, canvas, model_matrix);
        //}
    }

    pub(crate) fn render(&mut self, processor: &impl RenderPass3D) {
        for i in 0..=self.current {
            self.batches[i as usize].render(processor);
        }
        self.current = 0;
        self.previous = -1;
    }
}
