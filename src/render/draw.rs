use std::ops::Deref;
use std::rc::Rc;
use cgmath::{Vector4, Zero};
use crate::assets::{ReadableAssetManager, SemiAutomaticAssetManager};
use crate::render::batch::BatchController2D;

use crate::render::color::{Color, RGB};
use crate::render::shared::Window;

pub struct Draw2D<Win: Window> {
    window: Rc<Win>,
    canvas_coords: Vector4<u16>,
    color: Color<RGB, u8>,
    ctrl: BatchController2D
}

impl<Win: Window> Draw2D<Win> {
    pub(in crate::render) fn new(window: Rc<Win>, assets: &Rc<SemiAutomaticAssetManager>) -> Self {
        Draw2D {
            window,
            canvas_coords: Vector4::zero(),
            color: Color::<RGB, u8>::white(),
            ctrl: BatchController2D::new(assets.get_shader("default"), 10000)
        }
    }

    pub fn color(&mut self, color: &Color<RGB, u8>) {
        self.color.copy_of(color);
    }

    pub fn rgba(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.color.set(r, g, b, a);
    }

    pub fn triangleshapeorsmthlikethatanywaysaddthattothedrawqueuerightnowpleasecuziwannaseeanamazingtriangleandgreetitwithhellotriangle(&self) {

    }
}