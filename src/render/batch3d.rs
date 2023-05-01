use std::cell::RefCell;
use std::ops::{DerefMut, IndexMut};
use std::rc::Rc;
use glam::Mat4;
use mvutils::utils::RcMut;

use crate::render::model::{Model, TextureType};
use crate::render::shared::{RenderProcessor3D, Shader, Texture};
use crate::render::{MAX_TEXTURES, TEXTURE_LIMIT};

pub mod batch_layout_3d {
    use crate::render::batch2d::FLOAT_BYTES;

    //shared
    pub(crate) const POSITION_SIZE: u16 = 3;
    pub(crate) const MATERIAL_ID_SIZE: u16 = 1;

    //batch only
    pub(crate) const CANVAS_COORDS_SIZE: u16 = 4;
    pub(crate) const CANVAS_DATA_SIZE: u16 = 2;
    pub(crate) const MODEL_MATRIX_SIZE: u16 = 16;

    //vertex sizes
    pub(crate) const MODEL_VERTEX_SIZE_FLOATS: u16 = POSITION_SIZE + MATERIAL_ID_SIZE;
    pub(crate) const MODEL_VERTEX_SIZE_BYTES: u16 = MODEL_VERTEX_SIZE_FLOATS * FLOAT_BYTES;
    pub(crate) const BATCH_VERTEX_SIZE_FLOATS: u16 = POSITION_SIZE + MATERIAL_ID_SIZE + CANVAS_COORDS_SIZE + CANVAS_DATA_SIZE + MODEL_MATRIX_SIZE;
    pub(crate) const BATCH_VERTEX_SIZE_BYTES: u16 = BATCH_VERTEX_SIZE_FLOATS * FLOAT_BYTES;

    pub(crate) const POSITION_OFFSET: u16 = 0;
    pub(crate) const POSITION_OFFSET_BYTES: u16 = POSITION_OFFSET * FLOAT_BYTES;
    pub(crate) const MATERIAL_ID_OFFSET: u16 = POSITION_SIZE;
    pub(crate) const MATERIAL_ID_OFFSET_BYTES: u16 = MATERIAL_ID_OFFSET * FLOAT_BYTES;
    pub(crate) const CANVAS_COORDS_OFFSET: u16 = MATERIAL_ID_SIZE + MATERIAL_ID_OFFSET;
    pub(crate) const CANVAS_COORDS_OFFSET_BYTES: u16 = CANVAS_COORDS_OFFSET * FLOAT_BYTES;
    pub(crate) const CANVAS_DATA_OFFSET: u16 = CANVAS_COORDS_SIZE + CANVAS_COORDS_OFFSET;
    pub(crate) const CANVAS_DATA_OFFSET_BYTES: u16 = CANVAS_DATA_OFFSET * FLOAT_BYTES;
    pub(crate) const MODEL_MATRIX_OFFSET: u16 = CANVAS_DATA_SIZE + CANVAS_DATA_OFFSET;
    pub(crate) const MODEL_MATRIX_OFFSET_BYTES: u16 = MODEL_MATRIX_OFFSET * FLOAT_BYTES;

}

struct Batch3D {
    data: Vec<f32>,
    indices: Vec<u32>,
    textures: [Option<Rc<RefCell<Texture>>>; TEXTURE_LIMIT as usize + 1],
    tex_ids: [u32; TEXTURE_LIMIT as usize],
    size: u32,
    vert_count: u32,
    obj_count: u32,
    next_tex: u32,
    full: bool,
    full_tex: bool,
}

impl Batch3D {
    fn new(size: u32) -> Self {
        Batch3D {
            data: Vec::with_capacity(size as usize),
            indices: Vec::with_capacity(size as usize * 6),
            textures: [0; TEXTURE_LIMIT as usize + 1].map(|_| None),
            tex_ids: [0; TEXTURE_LIMIT as usize],
            size,
            vert_count: 0,
            obj_count: 0,
            next_tex: 0,
            full: false,
            full_tex: false
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

    fn is_full_tex_for(&self, amount: u32) -> bool {
        self.next_tex + amount < unsafe { MAX_TEXTURES }
    }

    fn add_texture(&mut self, texture: Rc<RefCell<Texture>>) -> u32 {
        if self.full_tex {
            return 0;
        }

        for i in 0..unsafe { MAX_TEXTURES as usize } {
            if let Some(tex) = &self.textures[i] {
                if tex.borrow().get_id() == texture.borrow().get_id() {
                    return i as u32 + 1;
                }
            }
        }

        self.textures[self.next_tex as usize] = Some(texture);
        self.tex_ids[self.next_tex as usize] = self.next_tex;
        self.next_tex += 1;

        if self.next_tex > unsafe { MAX_TEXTURES } {
            self.full_tex = true;
        }

        self.next_tex
    }

    fn push_model(&mut self, model: RcMut<Model>, canvas: [f32; 6], model_matrix: Mat4) {

    }

    fn render(&self, render_processor: &impl RenderProcessor3D, shader: &mut Shader) {

    }
}

struct Model3D {
    model: RcMut<Model>,
    canvas: [f32; 6],
    model_matrix: Mat4,
}

impl Model3D {
    fn new(model: RcMut<Model>, canvas: [f32; 6], model_matrix: Mat4) -> Self {
        Model3D {
            model,
            canvas,
            model_matrix,
        }
    }

    fn render(&self, render_processor: &impl RenderProcessor3D, shader: &mut Shader) {

    }
}

enum RenderType3D {
    Batch(Batch3D),
    Model(Model3D)
}

impl RenderType3D {
    fn is_model(&self) -> bool {
        match self {
            RenderType3D::Model(_) => true,
            _ => false
        }
    }

    fn is_batch(&self) -> bool {
        match self {
            RenderType3D::Batch(_) => true,
            _ => false
        }
    }

    fn get_model(&mut self) -> &mut Model3D {
        match self {
            RenderType3D::Model(model) => model,
            _ => unreachable!()
        }
    }

    fn get_batch(&mut self) -> &mut Batch3D {
        match self {
            RenderType3D::Batch(batch) => batch,
            _ => unreachable!()
        }
    }

    fn render(&self, render_processor: &impl RenderProcessor3D, batch_shader: &mut Shader, model_shader: &mut Shader) {
        match self {
            RenderType3D::Model(model) => model.render(render_processor, model_shader),
            RenderType3D::Batch(batch) => batch.render(render_processor, batch_shader)
        }
    }
}

//data

pub struct Vertex3D {
    data: [f32; batch_layout_3d::BATCH_VERTEX_SIZE_FLOATS as usize],
}

impl Vertex3D {
    pub fn new() -> Self {
        Vertex3D {
            data: [0.0; batch_layout_3d::BATCH_VERTEX_SIZE_FLOATS as usize]
        }
    }

    pub(crate) fn set(&mut self, data: [f32; batch_layout_3d::BATCH_VERTEX_SIZE_FLOATS as usize]) {
        self.data = data;
    }

    pub(crate) fn set_data(&mut self, x: f32, y: f32, z: f32, material_id: u16, canvas: [f32; 6], model_matrix: Mat4) {
        let mat = model_matrix.to_cols_array();
        self.set([x, y, z, material_id as f32,
            canvas[0],
            canvas[1],
            canvas[2],
            canvas[3],
            canvas[4],
            canvas[5],
            mat[0],
            mat[1],
            mat[2],
            mat[3],
            mat[4],
            mat[5],
            mat[6],
            mat[7],
            mat[8],
            mat[9],
            mat[10],
            mat[11],
            mat[12],
            mat[13],
            mat[14],
            mat[15]
        ]);
    }
}

//controller

pub(crate) struct BatchController3D {
    batches: Vec<RenderType3D>,
    batch_limit: u32,
    model_limit: u32,
    batch_shader: Rc<RefCell<Shader>>,
    default_batch_shader: Rc<RefCell<Shader>>,
    model_shader: Rc<RefCell<Shader>>,
    default_model_shader: Rc<RefCell<Shader>>,
    current: u32,
}

impl BatchController3D {
    pub(crate) fn new(batch_shader: Rc<RefCell<Shader>>, model_shader: Rc<RefCell<Shader>>, batch_limit: u32) -> Self {
        assert!(batch_limit > 14);
        let mut batch = BatchController3D {
            batches: Vec::new(),
            batch_limit,
            model_limit: (batch_limit * 3) / 4,
            default_batch_shader: batch_shader.clone(),
            batch_shader,
            default_model_shader: model_shader.clone(),
            model_shader,
            current: 0,
        };
        batch.start();
        batch
    }

    fn start(&mut self) {
        self.batches.push(self.gen_batch());
    }

    fn gen_batch(&self) -> RenderType3D {
        RenderType3D::Batch(Batch3D::new(self.batch_limit))
    }

    fn next_batch(&mut self) {
        self.current += 1;
        if let Some(render_type) = self.batches.get(self.current as usize) {
            if render_type.is_model() {
                self.batches[self.current as usize] = self.gen_batch();
            }
        }
        else {
            self.batches.push(self.gen_batch());
        }
    }

    fn ensure_batch(&mut self, len: u32, textures: u32) {
        if self.batches[self.current as usize].is_model() {
            self.next_batch();
        }
        else {
            let batch = self.batches[self.current as usize].get_batch();
            if batch.is_full(len) || batch.is_full_tex_for(textures) {
                self.next_batch();
            }
        }
    }

    fn push_model(&mut self, model: RcMut<Model>, canvas: [f32; 6], model_matrix: Mat4) {
        let model = RenderType3D::Model(Model3D::new(model, canvas, model_matrix));
        self.current += 1;
        if self.batches.len() > self.current as usize {
            self.batches[self.current as usize] = model;
        }
        else {
            self.batches.push(model);
        }
    }

    pub(crate) fn add_model(&mut self, model: RcMut<Model>, canvas: [f32; 6], model_matrix: Mat4) {
        if model.borrow().is_simple_geometry() && model.borrow().vertex_count() <= self.model_limit {
            self.ensure_batch(model.borrow().vertex_count(), model.borrow().texture_count(TextureType::Geometry));
            self.batches[self.current as usize].get_batch().push_model(model, canvas, model_matrix);
        }
        else {
            self.push_model(model, canvas, model_matrix);
        }
    }

    pub(crate) fn render(&mut self, processor: &impl RenderProcessor3D) {
        self.batch_shader.borrow_mut().bind();
        for i in 0..self.current + 1 {
            self.batches[i as usize].render(processor, self.batch_shader.borrow_mut().deref_mut(), self.model_shader.borrow_mut().deref_mut());
        }
        self.current = 0;
    }

    pub(crate) fn set_batch_shader(&mut self, shader: Rc<RefCell<Shader>>) {
        self.batch_shader = shader;
    }

    pub(crate) fn reset_batch_shader(&mut self) {
        self.set_batch_shader(self.default_batch_shader.clone());
    }

    pub(crate) fn set_model_shader(&mut self, shader: Rc<RefCell<Shader>>) {
        self.model_shader = shader;
    }

    pub(crate) fn reset_model_shader(&mut self) {
        self.set_model_shader(self.default_model_shader.clone());
    }
}