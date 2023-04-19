use std::cell::RefCell;
use std::rc::Rc;

use crate::render::batch::{BatchController2D, Vertex2D, VertexGroup};
use crate::render::color::{Color, RGB};
use crate::render::shared::{RenderProcessor2D, Shader};

pub struct Draw2D {
    canvas_coords: [f32; 4],
    color: Color<RGB, u8>,
    batch: BatchController2D,
    vertices: VertexGroup<Vertex2D>,
}

impl Draw2D {
    pub(crate) fn new(shader: Rc<RefCell<Shader>>, width: i32, height: i32) -> Self {
        Draw2D {
            canvas_coords: [0.0, 0.0, width as f32, height as f32],
            color: Color::<RGB, u8>::white(),
            batch: BatchController2D::new(shader, 10000),
            vertices: VertexGroup::new(),
        }
    }

    pub(crate) fn render(&mut self, processor: &impl RenderProcessor2D) {
        self.batch.render(processor)
    }

    pub(crate) fn reset_canvas(&mut self, width: i32, height: i32) {
        self.canvas_coords = [0.0, 0.0, width as f32, height as f32];
    }

    pub fn color(&mut self, color: &Color<RGB, u8>) {
        self.color.copy_of(color);
    }

    pub fn rgba(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.color.set(r, g, b, a);
    }

    pub fn tri(&mut self) {
        self.vertices.get_mut(0).set([100.0, 100.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 800.0, 600.0, 0.0, 0.0, 0.0]);
        self.vertices.get_mut(1).set([200.0, 100.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 800.0, 600.0, 0.0, 0.0, 0.0]);
        self.vertices.get_mut(2).set([150.0, 200.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 800.0, 600.0, 0.0, 0.0, 0.0]);
        self.vertices.set_len(3);
        self.batch.add_vertices(&self.vertices);
    }
}