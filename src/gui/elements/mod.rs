use std::sync::{Arc, RwLock};
use mvcore_proc_macro::gui_element;
use crate::gui::styles::Style;
use crate::render::draw2d::Draw2D;

#[gui_element]
pub struct GuiElement {
    pub style: Style,
    pub parent: Option<Arc<RwLock<dyn IGuiElement>>>,
}

pub trait Drawable {
    fn draw(&self, ctx: &mut Draw2D) {

    }
}

pub trait IGuiElement: Drawable {
    fn style(&self) -> &Style;
    fn style_mut(&mut self) -> &mut Style;
}