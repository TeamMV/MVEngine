use crate::gui::styles::Style;
use crate::render::draw2d::Draw2D;
use mvcore_proc_macro::{gui_element, gui_element_trait};
use std::sync::{Arc, RwLock};

#[gui_element]
pub struct GuiElementImpl {
    //pub style: Style,
    //pub parent: Option<Arc<RwLock<dyn GuiElement>>>,
}

pub trait GuiElementCallbacks {
    fn draw(&self, ctx: &mut Draw2D) {}
}

gui_element_trait!();

//pub trait GuiElement: GuiElementCallbacks {
//    //fn style(&self) -> &Style;
//    //fn style_mut(&mut self) -> &mut Style;
//}
