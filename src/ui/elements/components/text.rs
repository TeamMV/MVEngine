use crate::rendering::RenderContext;
use crate::ui::elements::{UiElementState, UiElementStub};
use crate::ui::geometry::SimpleRect;
use ropey::Rope;
use crate::ui::styles::UiStyle;

#[derive(Clone)]
pub struct TextBody {}

impl TextBody {
    pub fn draw(
        &self,
        style: &UiStyle,
        state: &UiElementState,
        s: &Rope,
        ctx: &mut impl RenderContext,
        crop_area: &SimpleRect,
    ) {
        //TODO
    }
}
