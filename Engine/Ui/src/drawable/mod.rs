pub mod color;

use mve2d::renderer2d::GameRenderer2D;
use crate::drawable::color::ColorDrawable;
use crate::styles::{Dimension, Location};

pub enum Drawable {
    Color(ColorDrawable),
}

pub trait DrawableCallbacks {
    fn draw(&mut self, location: Location<i32>, renderer: &mut GameRenderer2D);
}
