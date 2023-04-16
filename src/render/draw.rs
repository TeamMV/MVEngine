use std::ops::Deref;
use cgmath::{Vector4, Zero};

use crate::render::color::{Color, RGB};
use crate::render::shared::Window;

pub struct Draw2D<Win: Window> {
    window: Win,
    canvas_coords: Vector4<u16>,
    color: Color<RGB, u8>,
}

impl<Win: Window> Draw2D<Win> {
    pub(in crate::render) fn new(window: Win) -> Self {
        Draw2D {
            window,
            canvas_coords: Vector4::zero(),
            color: Color::<RGB, u8>::white(),
        }
    }

    pub fn color(&mut self, color: &Color<RGB, u8>) {
        self.color.copy_of(color);
    }

    pub fn rgba(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.color.set(r, g, b, a);
    }


}