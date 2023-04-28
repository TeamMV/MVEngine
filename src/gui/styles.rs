use alloc::rc::Rc;
use std::ops::Deref;

use mvutils::deref;
use mvutils::screen::{Measurement, Measurements};
use mvutils::utils::{RcMut, XTraFMath};

use crate::gui::components::{GuiElement, GuiElementInfo};
use crate::render::color::{Fmt, Gradient, RGB};
use crate::render::draw::Draw2D;
use crate::render::text::TypeFace;

#[derive(Default)]
pub struct GuiStyle {
    pub background_color: GuiValue<Gradient<RGB, f32>>,
    pub foreground_color: GuiValue<Gradient<RGB, f32>>,
    pub text_color: GuiValue<Gradient<RGB, f32>>,
    pub text_chroma: GuiValue<bool>,
    pub text_size: GuiValue<i32>,
    pub font: GuiValue<Option<Rc<TypeFace>>>,
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
    pub z_index: GuiValue<u16>
}

#[derive(Copy, Clone, Default)]
pub enum ViewState {
    #[default]
    There,
    None,
    Gone
}

#[derive(Copy, Clone, Default)]
pub enum Positioning {
    #[default]
    Relative,
    Absolute,
    Sticky(*const dyn FnMut(i32, i32, i32, i32) -> bool)
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Default)]
pub enum BorderStyle {
    #[default]
    Square,
    Round,
    Triangle
}

pub trait GuiValueValue<T> {
    fn compute_measurement(&self, dpi: f32, mes: &Measurement) -> T;
    fn compute_percentage_value(&self, total: T, percentage: u8) -> T;
}

impl GuiValueValue<(i32, i32)> for (i32, i32) {
    fn compute_measurement(&self, dpi: f32, mes: &Measurement) -> (i32, i32) {
        (Measurements::compute(dpi, self.0 as f32, mes) as i32, Measurements::compute(dpi, self.1 as f32, mes) as i32)
    }

    fn compute_percentage_value(&self, total: (i32, i32), percentage: u8) -> (i32, i32) {
        ((percentage as f32).value(total.0 as f32) as i32, (percentage as f32).value(total.1 as f32) as i32)
    }
}

macro_rules! impl_gvv_prim {
    ($($typ:ty),*) => {
        $(
            impl GuiValueValue<$typ> for $typ {
                fn compute_measurement(&self, dpi: f32, mes: &Measurement) -> $typ {
                    Measurements::compute(dpi, *self as f32, mes) as $typ
                }

                fn compute_percentage_value(&self, total: $typ, percentage: u8) -> $typ {
                    (percentage as f32).value(total as f32) as $typ
                }
            }
        )*
    };
}

impl_gvv_prim!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64);
macro_rules! impl_unreachable_gvv {
    ($($typ:ty),*) => {
        $(
            impl GuiValueValue<$typ> for $typ {
                fn compute_measurement(&self, dpi: f32, mes: &Measurement) -> $typ {
                    unreachable!()
                }

                fn compute_percentage_value(&self, total: $typ, percentage: u8) -> $typ {
                    unreachable!()
                }
            }
        )*
    };
}

impl_unreachable_gvv!(Gradient<RGB, f32>, ViewState, Positioning, BorderStyle, Option<Rc<TypeFace>>, bool);

pub struct GuiValueComputeSupply {
    pub dpi: f32,
    pub parent: Option<RcMut<GuiElement>>
}

impl GuiValueComputeSupply {
    pub fn new(dpi: f32, parent: Option<RcMut<GuiElement>>) -> Self {
        GuiValueComputeSupply {
            dpi,
            parent
        }
    }

    fn get_dpi(&self) -> f32 {
        self.dpi
    }

    fn get_parent(&self) -> &Option<RcMut<GuiElement>> {
        &self.parent
    }
}

pub enum GuiValue<T: Clone + GuiValueValue<T>> {
    Just(T),
    Measurement(T, Measurement),
    Percentage(T, u8),
    Inherit(),
    Clone(&'static GuiStyle)
}

impl<T: Clone + GuiValueValue<T>> GuiValue<T> {
    pub fn unwrap<F>(&self, compute_supply: &GuiValueComputeSupply, resolve_supply: F) -> T where F: FnMut(&GuiStyle) -> GuiValue<T> {
        match self {
            GuiValue::Just(t) => {t.clone()},
            GuiValue::Measurement(t, mes) => {t.compute_measurement(compute_supply.get_dpi(), mes)},
            GuiValue::Percentage(t, perc) => {t.compute_percentage_value(t.clone(), perc.clone())},
            GuiValue::Inherit() => {resolve_supply(&compute_supply.get_parent().expect("Set 'Inherit' on element without parent!").style).unwrap(compute_supply, resolve_supply)},
            GuiValue::Clone(other) => {resolve_supply(other).unwrap(compute_supply, resolve_supply)}
        }
    }
}

impl<T: Default + Clone + GuiValueValue<T>> Default for GuiValue<T> {
    fn default() -> Self {
        GuiValue::Just(T::default())
    }
}

#[macro_export]
macro_rules! resolve {
    ($info:expr, $prop:ident) => {
        $info.style.$prop.unwrap(&$info.compute_supply, |s| {s.$prop})
    };
    ($info:ident, $prop:ident) => {
        $info.style.$prop.unwrap(&$info.compute_supply, |s| {s.$prop})
    };
}