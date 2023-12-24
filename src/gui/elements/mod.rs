use crate::gui::styles::{GuiValue, Position, ResCon, Style};
use crate::render::draw2d::DrawContext2D;
use crate::{resolve};
use mvcore_proc_macro::{gui_element, gui_element_trait};
use mvutils::unsafe_utils::Unsafe;
use mvutils::utils::{Recover, RwArc, TetrahedronOp};
use std::sync::{Arc, RwLock};
use crate::gui::styles::Origin::{Center, Custom};

#[gui_element]
pub struct GuiElementImpl {}

impl GuiElementImpl {}

pub trait GuiElementCallbacks {
    fn draw(&self, ctx: &mut DrawContext2D)
    where
        Self: GuiElement;

    fn compute_values(&mut self, ctx: &mut DrawContext2D)
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

        let origin = resolve!(self, origin);
        let position = resolve!(self, position);

        let mut x: i32 = 0;
        let mut y: i32 = 0;
        if position == Position::Absolute {
            x = resolve!(self, x);
            y = resolve!(self, y);
        } else if position == Position::Relative {
            x = self.x();
            y = self.y();
        }

        self.set_border_x(x + margins[2]);
        self.set_border_y(y + margins[1]);

        if let Custom(ox, oy) = origin {
            self.set_x(x + ox);
            self.set_y(y + oy);
        } else {
            if let Center = origin {
                self.set_x(x + self.width() / 2);
                self.set_y(y + self.height() / 2);
            } else {
                self.set_x(origin.is_right().yn(x - self.width(), x));
                self.set_y(origin.is_left().yn(y - self.height(), y));
            }
        }
    }
}

impl GuiElementCallbacks for GuiElementImpl {
    fn draw(&self, ctx: &mut DrawContext2D)
    where
        Self: GuiElement,
    {
        todo!()
    }
}

gui_element_trait!();
