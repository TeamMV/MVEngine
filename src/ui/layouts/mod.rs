use crate::ui::context::UiContext;
use crate::ui::elements::Element;

pub mod uniqueselect;
pub mod flow;

pub trait Adapter {
    fn create_element(&mut self, ctx: UiContext) -> Option<Element>;
}