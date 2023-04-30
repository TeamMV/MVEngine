use std::cell::RefCell;
use std::ops::{DerefMut, IndexMut};
use std::rc::Rc;
use mvutils::utils::RcMut;

use crate::render::model::Model;
use crate::render::shared::{Shader, Texture};

pub mod batch_layout_2d {
    use crate::render::batch2d::FLOAT_BYTES;


}

struct Batch3D {
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

impl Batch3D {
    fn new(size: u32) -> Self {
        Batch3D {
            data: Vec::with_capacity(size as usize),
            indices: Vec::with_capacity(size as usize * 6),
            textures: [0; 17].map(|_| None),
            tex_ids: [0; 16],
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
}

enum RenderType3D {
    Batch(Batch3D),
    Model(RcMut<Model>)
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

    fn get_model(&self) -> RcMut<Model> {
        match self {
            RenderType3D::Model(model) => model.clone(),
            _ => unreachable!()
        }
    }

    fn get_batch(&self) -> &Batch3D {
        match self {
            RenderType3D::Batch(batch) => batch,
            _ => unreachable!()
        }
    }
}

pub(crate) struct BatchController3D {
    batches: Vec<RenderType3D>,
    batch_limit: u32,
    model_limit: u32,
    shader: Rc<RefCell<Shader>>,
    default_shader: Rc<RefCell<Shader>>,
    current: u32,
}

impl BatchController3D {
    pub(crate) fn new(shader: Rc<RefCell<Shader>>, batch_limit: u32) -> Self {
        assert!(batch_limit > 14);
        let mut batch = BatchController3D {
            batches: Vec::new(),
            batch_limit,
            model_limit: (batch_limit * 3) / 4,
            default_shader: shader.clone(),
            shader,
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

    fn ensure_batch(&mut self, len: u32) {
        if self.batches[self.current as usize].is_model() {
            self.next_batch();
        }
        else {
            let batch = self.batches[self.current as usize].get_batch();
            if batch.is_full(len) {
                self.next_batch();
            }
        }
    }

    fn push_model(&mut self, model: RcMut<Model>) {
        self.current += 1;
        if self.batches.len() > self.current as usize {
            self.batches[self.current as usize] = RenderType3D::Model(model);
        }
        else {
            self.batches.push(RenderType3D::Model(model));
        }
    }

    pub(crate) fn add_model(&mut self, model: RcMut<Model>) {
        if model.borrow().vertex_count() > self.model_limit {
            self.push_model(model);
            return;
        }
        self.ensure_batch(model.borrow().vertex_count());
    }
}