mod text;

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

impl GuiElementImpl {}

pub trait DrawComponentBody {
    fn draw_component_body(&self) where Self: GuiElement {

    }
}

pub trait GuiElementCallbacks {
    fn draw(&mut self, ctx: &mut DrawContext2D);
}

impl GuiElementCallbacks for GuiElementImpl {
    fn draw(&mut self, ctx: &mut DrawContext2D) {

    }
}

gui_element_trait!();
