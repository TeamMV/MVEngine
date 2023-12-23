use crate::gui::styles::{GuiValue, ResCon, Style};
use crate::render::draw2d::Draw2D;
use crate::resolve;
use mvcore_proc_macro::{gui_element, gui_element_trait};
use mvutils::unsafe_utils::Unsafe;
use mvutils::utils::{Recover, RwArc};
use std::sync::{Arc, RwLock};

#[gui_element]
pub struct GuiElementImpl {}

impl GuiElementImpl {}

pub trait GuiElementCallbacks {
    fn draw(&self, ctx: &mut Draw2D)
    where
        Self: GuiElement;

    fn compute_values(&mut self, ctx: &mut Draw2D)
    where
        Self: GuiElement,
    {
        self.resolve_context_mut().set_dpi(ctx.dpi());
        let mut paddings: [i32; 4] = [0; 4];
        paddings[0] = resolve!(self, padding.top).unwrap();
        paddings[1] = resolve!(self, padding.bottom).unwrap();
        paddings[2] = resolve!(self, padding.left).unwrap();
        paddings[3] = resolve!(self, padding.right).unwrap();

        let mut margins: [i32; 4] = [0; 4];
        margins[0] = resolve!(self, margin.top).unwrap();
        margins[1] = resolve!(self, margin.bottom).unwrap();
        margins[2] = resolve!(self, margin.left).unwrap();
        margins[3] = resolve!(self, margin.right).unwrap();

        if self.style().width.is_set() {
            self.set_content_width(resolve!(self, width).unwrap());
        }

        if self.style().height.is_set() {
            self.set_content_height(resolve!(self, height).unwrap());
        }

        let bounding_width = self.content_width() + paddings[2] + paddings[3];
        let width = bounding_width + margins[2] + margins[3];
        let bounding_height = self.content_height() + paddings[0] + paddings[1];
        let height = bounding_width + margins[0] + margins[1];

        self.set_bounding_width(bounding_width);
        self.set_bounding_height(bounding_height);
        self.set_width(width);
        self.set_height(height);

        let origin = resolve!(self, origin).unwrap();
    }
}

impl GuiElementCallbacks for GuiElementImpl {
    fn draw(&self, ctx: &mut Draw2D)
    where
        Self: GuiElement,
    {
        todo!()
    }
}

gui_element_trait!();
