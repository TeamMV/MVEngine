use mvutils::screen::{Measurement, Measurements};
use mvutils::utils::XTraFMath;
use crate::render::color::{Gradient, RGB};
use crate::render::draw::Draw2D;
use crate::render::text::{Font, TypeFace};
use crate::vgui_v5::components::{GuiElement, GuiElementInfo};

pub struct GuiStyle {
    pub background_color: GuiValue<Gradient<RGB, f32>>,
    pub foreground_color: GuiValue<Gradient<RGB, f32>>,
    pub text_color: GuiValue<Gradient<RGB, f32>>,
    pub text_chroma: GuiValue<bool>,
    pub text_size: GuiValue<i32>,
    pub font: GuiValue<TypeFace>,
    //left, right, bottom, top
    pub margin: [GuiValue<f32>; 4],
    pub padding: [GuiValue<f32>; 4],
    pub border_width: GuiValue<i32>,
    pub border_radius: GuiValue<i32>,
    pub border_style: GuiValue<BorderStyle>,
    pub border_color: GuiValue<Gradient<RGB, f32>>,
    pub view_state: GuiValue<ViewState>,
    pub positioning: GuiValue<Positioning>,
    pub width: GuiValue<i32>,
    pub height: GuiValue<i32>,
    pub left: GuiValue<i32>,
    pub right: GuiValue<i32>,
    pub top: GuiValue<i32>,
    pub bottom: GuiValue<i32>,
    pub rotation: GuiValue<f32>,
    pub rotation_center: GuiValue<(i32, i32)>,
}

pub enum ViewState {
    There,
    None,
    Gone
}

pub enum Positioning {
    Relative,
    Absolute,
    Sticky(dyn FnMut(i32, i32, i32, i32) -> bool)
}

pub enum BorderStyle {
    Round,
    Triangle
}

pub enum GuiValue<T> {
    Just(T),
    Measurement(T, Measurement),
    Percentage(T, u8),
    Inherit(),
    Clone(GuiStyle)
}

impl<T> GuiValue<T> {
    pub fn unwrap(&self, draw: &Draw2D, element_info: &GuiElementInfo, name_supplier: Box<dyn FnMut(&GuiStyle) -> &GuiValue<T>>) -> T {
        match name_supplier(self) {
            GuiValue::Just(t) => {t},
            GuiValue::Measurement(v, m) => {Measurements::compute(draw.dpi(), v as f32, &m)},
            GuiValue::Percentage(v, p) => {(v as f32).value(p as f32)},
            GuiValue::Inherit() => {name_supplier(self).unwrap(draw, element_info, name_supplier)},
            GuiValue::Clone(other) => {name_supplier(&other).unwrap(draw, element_info, name_supplier)}
        }
    }
}