use crate::render::draw2d::DrawContext2D;
use crate::ui::prelude::*;

#[ui_element]
pub struct UiLabel {
    text: String,
}

impl UiElementCallbacks for UiLabel {
    fn draw(&mut self, ctx: &mut DrawContext2D) {}
}
