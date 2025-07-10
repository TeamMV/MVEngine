use crate::rendering::RenderContext;
use crate::ui::elements::UiElementStub;
use crate::ui::geometry::SimpleRect;
use ropey::Rope;

#[derive(Clone)]
pub struct TextBody {}

impl TextBody {
    pub fn draw<E: UiElementStub + 'static>(
        &self,
        elem: &mut E,
        s: &Rope,
        ctx: &mut impl RenderContext,
        crop_area: &SimpleRect,
    ) {
        //TODO
    }
}
