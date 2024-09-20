pub mod color;

use crate::drawable::color::ColorDrawable;
use crate::styles::{Dimension, Location};
use mve2d::renderer2d::GameRenderer2D;

pub enum Drawable {
    Color(ColorDrawable),
}

pub trait DrawableCallbacks {
    fn draw(&mut self, location: Location<i32>, renderer: &mut GameRenderer2D);
}
