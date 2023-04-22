use std::cell::RefCell;
use std::rc::Rc;

use crate::render::batch::{BatchController2D, Vertex2D, VertexGroup};
use crate::render::color::{Color, RGB};
use crate::render::shared::{RenderProcessor2D, Shader};

pub struct Draw2D {
    canvas: [f32; 6],
    size: [f32; 2],
    color: Color<RGB, f32>,
    batch: BatchController2D,
    vertices: VertexGroup<Vertex2D>,
}

impl Draw2D {
    pub(crate) fn new(shader: Rc<RefCell<Shader>>, width: i32, height: i32) -> Self {
        Draw2D {
            canvas: [0.0, 0.0, width as f32, height as f32, 0.0, 0.0],
            size: [width as f32, height as f32],
            color: Color::<RGB, f32>::white(),
            batch: BatchController2D::new(shader, 10000),
            vertices: VertexGroup::new(),
        }
    }

    pub(crate) fn render(&mut self, processor: &impl RenderProcessor2D) {
        self.batch.render(processor)
    }

    pub(crate) fn resize(&mut self, width: i32, height: i32) {
        self.size[0] = width as f32;
        self.size[1] = height as f32;
    }

    pub fn reset_canvas(&mut self) {
        self.canvas[0] = 0.0;
        self.canvas[1] = 0.0;
        self.canvas[2] = self.size[0];
        self.canvas[3] = self.size[1];
        self.canvas[4] = 0.0;
        self.canvas[5] = 0.0;
    }

    pub fn style_canvas(&mut self, style: CanvasStyle, radius: f32) {
        self.canvas[4] = style.id();
        self.canvas[5] = radius;
    }

    pub fn set_canvas(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.canvas[0] = x as f32;
        self.canvas[1] = y as f32;
        self.canvas[2] = width as f32;
        self.canvas[3] = height as f32;
    }

    pub fn color(&mut self, color: &Color<RGB, f32>) {
        self.color.copy_of(color);
    }

    pub fn rgba(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.color.normalize(r, g, b, a);
    }

    pub fn raw_rgba(&mut self, r: f32, g: f32, b: f32, a: f32) {
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

pub enum CanvasStyle {
    Square,
    Triangle,
    Round
}

impl CanvasStyle {
    pub(crate) fn id(&self) -> f32 {
        match self {
            CanvasStyle::Square => 0.0,
            CanvasStyle::Triangle => 1.0,
            CanvasStyle::Round => 2.0,
        }
    }
}