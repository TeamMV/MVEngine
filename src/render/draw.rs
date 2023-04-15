use std::ops::Deref;
use crate::math::vector::Vector4;
use crate::render::color::{Color, RGB};
use crate::render::shared::Window;

pub struct DrawContext2D<'l, Win: Window> {
    window: Win,
    canvas_coords: &'l Vector4<u16>,
    //color: &'l Color<RGB, u8>
}

impl<Win: Window> DrawContext2D<'_, Win> {
    pub fn new(window: Win) -> Self {
        DrawContext2D {
            window,
            canvas_coords: &Vector4 {
                x: 0,
                y: 0,
                z: 0,
                w: 0,
            },
            //color: &Color::<RGB, u8>::white()
            //TODO: get this /\ owning problem fixed
        }
    }

    pub fn color(&mut self, color: &Color<RGB, u8>) {
        //self.color = color.clone();
    }

    pub fn rgba(&mut self, r: u8, g: u8, b: u8, a: u8) {
        //self.color.set(r, g, b, a);
    }


}