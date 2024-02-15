use crate::gui::element_file::*;
use crate::render::draw2d::DrawContext2D;

#[gui_element]
pub struct GuiLabel {
    text: String,
}

impl GuiElementCallbacks for GuiLabel {
    fn draw(&mut self, ctx: &mut DrawContext2D) {}
}
