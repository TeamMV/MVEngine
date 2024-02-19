mod text;

use crate::gui::element_file::Background;
use crate::gui::styles::{GuiValue, Origin, Position, ResCon, Style};
use crate::gui::Sides;
use crate::render::draw2d::DrawContext2D;
use crate::resolve;
use mvcore_proc_macro::{gui_element, gui_element_trait};
use mvutils::unsafe_utils::Unsafe;
use mvutils::utils::{Recover, RwArc, TetrahedronOp};
use std::sync::{Arc, RwLock};

#[gui_element]
pub struct GuiElementImpl {}

impl GuiElementImpl {
    pub fn test() -> Self {
        Self {
            id: "test".to_string(),
            x: 100,
            y: 100,
            border_x: 0,
            border_y: 0,
            content_x: 0,
            content_y: 0,
            style: Default::default(),
            parent: None,
            resolve_context: ResCon { dpi: 0.0 },
            content_width: 300,
            content_height: 200,
            bounding_width: 0,
            bounding_height: 0,
            width: 0,
            height: 0,
            origin_x: 0,
            origin_y: 0,
            paddings: Sides::same(0),
            margins: Sides::same(0),
            background: None,
        }
    }
}

pub trait DrawComponentBody {
    fn draw_component_body(&self, ctx: &mut DrawContext2D) {

    }
}

pub trait GuiElementCallbacks {
    fn draw(&mut self, ctx: &mut DrawContext2D);
}

impl GuiElementCallbacks for GuiElementImpl {
    fn draw(&mut self, ctx: &mut DrawContext2D) {}
}

gui_element_trait!();
