pub mod color;
mod atlas;

use crate::render::draw2d::DrawContext2D;
use crate::ui::drawable::color::ColorDrawable;
use crate::ui::styles::{Dimension, Location};

pub enum Drawable {
    Color(ColorDrawable),
}

pub trait DrawableCallbacks {
    fn draw(&mut self, location: Location<i32>, ctx: &mut DrawContext2D);
}
