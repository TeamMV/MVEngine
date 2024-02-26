mod text;

use crate::render::color::RgbColor;
use crate::render::draw2d::DrawContext2D;
use crate::resolve;
use crate::ui::prelude::Background;
use crate::ui::styles::{Origin, Position, ResCon, Style, UiValue};
use crate::ui::Sides;
use mvcore_proc_macro::{ui_element, ui_element_trait};
use mvutils::unsafe_utils::Unsafe;
use mvutils::utils::{Recover, RwArc, TetrahedronOp};
use std::sync::{Arc, RwLock};

#[ui_element]
pub struct UiElementImpl {}

impl UiElementImpl {
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
    fn draw_component_body(&self, ctx: &mut DrawContext2D)
    where
        Self: UiElement,
    {
        todo!()
    }
}

pub trait UiElementCallbacks {
    fn draw(&mut self, ctx: &mut DrawContext2D);
}

impl UiElementCallbacks for UiElementImpl {
    fn draw(&mut self, ctx: &mut DrawContext2D) {}
}

ui_element_trait!();
