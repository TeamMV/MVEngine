pub mod elements;
pub mod styles;
pub mod parsing;

pub mod element_file {
    pub use mvcore_proc_macro::gui_element;
    pub use crate::gui::elements::GuiElementCallbacks;
    pub use crate::gui::elements::GuiElement;
    pub use crate::gui::elements::DrawComponentBody;
    pub use crate::gui::styles::*;
    pub use std::sync::Arc;
    pub use crate::gui::Sides;
}

pub struct Sides {
    pub top: i32,
    pub bottom: i32,
    pub left: i32,
    pub right: i32,
}

impl Sides {
    pub fn copy_slice(&mut self, data: &[i32]) {
        self.top = data[0];
        self.bottom = data[1];
        self.left = data[2];
        self.right = data[3];
    }
}
