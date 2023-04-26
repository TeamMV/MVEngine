use alloc::rc::Rc;
use std::cell::RefCell;
use mvutils::deref;
use mvutils::screen::{Measurement, Measurements};
use mvutils::utils::{RcMut, XTraFMath};
use crate::render::color::{Gradient, RGB};
use crate::render::draw::Draw2D;
use crate::render::text::{Font, TypeFace};
use crate::gui::components::{GuiElement, GuiElementInfo};

#[derive(Default)]
pub struct GuiStyle {
    pub background_color: GuiValue<Gradient<RGB, f32>>,
    pub foreground_color: GuiValue<Gradient<RGB, f32>>,
    pub text_color: GuiValue<Gradient<RGB, f32>>,
    pub text_chroma: GuiValue<bool>,
    pub text_size: GuiValue<i32>,
    pub font: GuiValue<Rc<TypeFace>>,
    pub margin_left: GuiValue<i32>,
    pub margin_right: GuiValue<i32>,
    pub margin_bottom: GuiValue<i32>,
    pub margin_top: GuiValue<i32>,
    pub padding_left: GuiValue<i32>,
    pub padding_right: GuiValue<i32>,
    pub padding_bottom: GuiValue<i32>,
    pub padding_top: GuiValue<i32>,
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

#[derive(Copy, Clone)]
pub enum ViewState {
    There,
    None,
    Gone
}

#[derive(Copy, Clone)]
pub enum Positioning {
    Relative,
    Absolute,
    Sticky(*const dyn FnMut(i32, i32, i32, i32) -> bool)
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum BorderStyle {
    Round,
    Triangle
}

pub enum GuiValue<T> {
    Just(T),
    Measurement(T, Measurement),
    Percentage(T, u8),
    Inherit(),
    Clone(&'static GuiStyle)
}

macro_rules! impl_gv {
    ($($t:ty),*) => {
        $(
            impl GuiValue<$t> {
                pub fn unwrap<F>(&self, draw: RcMut<Draw2D>, element_info: &GuiElementInfo, mut name_supplier: F) -> $t where
                    F: FnMut(&GuiStyle) -> &GuiValue<$t> {
                        match self {
                        GuiValue::Just(t) => {deref!(t)},
                        GuiValue::Measurement(v, m) => {Measurements::compute(draw.borrow_mut().dpi(), deref!(v) as f32, &m) as $t},
                        GuiValue::Percentage(v, p) => {(deref!(v) as f32).value(deref!(p) as f32) as $t},
                        GuiValue::Inherit() => {name_supplier(&element_info.style).unwrap(draw.clone(), element_info, name_supplier)},
                        GuiValue::Clone(other) => {name_supplier(other).unwrap(draw.clone(), element_info, name_supplier)}
                        }//My funny ide doenst let me fix this formatting issue loool
                }
            }
        )*
    };
}

impl<T: Copy> GuiValue<T> {
    pub fn unwrapt<F>(&self, draw: RcMut<Draw2D>, element_info: &GuiElementInfo, mut name_supplier: F) -> T where
        F: FnMut(&GuiStyle) -> &GuiValue<T> {
        match self {
            GuiValue::Just(t) => {deref!(t)},
            GuiValue::Measurement(v, m) => {deref!(v)},
            GuiValue::Percentage(v, p) => {deref!(v)},
            GuiValue::Inherit() => {name_supplier(&element_info.style).unwrapt(draw.clone(), element_info, name_supplier)},
            GuiValue::Clone(other) => {name_supplier(other).unwrapt(draw.clone(), element_info, name_supplier)}
        }
    }
}

macro_rules! resolve {
    ($n:ident) => {self.info.style.$n.unwrapt(ctx.clone(), self.info(), |s| {&s.$n})};
    ($v:ident, $n:ident) => {$v.info.style.$n.unwrapt(ctx.clone(), $v.info(), |s| {&s.$n})};
    ($n:ident, $ign:expr) => {info.style.$n.unwrapt(ctx.clone(), info, |s| {&s.$n})};
    ($v:ident, $ign:expr, $n:ident) => {$v.style.$n.unwrapt(ctx.clone(), $v, |s| {&s.$n})};
}

pub(crate) use resolve;

impl_gv!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64);

impl<T: Default> Default for GuiValue<T> {
    fn default() -> Self {
        GuiValue::Just(T::default())
    }
}