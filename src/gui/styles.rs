use alloc::rc::Rc;
use std::ops::Deref;

use mvutils::deref;
use mvutils::screen::{Measurement};
use mvutils::utils::{RcMut, XTraFMath};

use crate::gui::components::{GuiElement, GuiElementInfo};
use crate::render::color::{Fmt, Gradient, RGB};
use crate::render::draw::Draw2D;
use crate::render::text::TypeFace;

pub struct GuiStyle {
    pub background_color: GuiValue<Gradient<RGB, f32>>,
    pub foreground_color: GuiValue<Gradient<RGB, f32>>,
    pub text_color: GuiValue<Gradient<RGB, f32>>,
    pub text_chroma: GuiValue<bool>,
    pub text_chroma_compress: GuiValue<f32>,
    pub text_chroma_tilt: GuiValue<f32>,
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

impl Default for GuiStyle {
    fn default() -> Self {
        GuiStyle {
            background_color: Default::default(),
            foreground_color: Default::default(),
            text_color: Default::default(),
            text_chroma: Default::default(),
            text_chroma_compress: GuiValue::Just(1.0),
            text_chroma_tilt: GuiValue::Just(-0.5),
            text_size: GuiValue::Just(20),
            font: Default::default(),
            margin_left: Default::default(),
            margin_right: Default::default(),
            margin_bottom: Default::default(),
            margin_top: Default::default(),
            padding_left: Default::default(),
            padding_right: Default::default(),
            padding_bottom: Default::default(),
            padding_top: Default::default(),
            border_width: Default::default(),
            border_radius: Default::default(),
            border_style: Default::default(),
            border_color: Default::default(),
            view_state: Default::default(),
            positioning: Default::default(),
            width: Default::default(),
            height: Default::default(),
            left: Default::default(),
            right: Default::default(),
            top: Default::default(),
            bottom: Default::default(),
            rotation: Default::default(),
            rotation_center: Default::default(),
            z_index: Default::default(),
        }
    }
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
    fn compute_percentage_value_self(&self, percentage: u8) -> T;
}

impl GuiValueValue<(i32, i32)> for (i32, i32) {
    fn compute_measurement(&self, dpi: f32, mes: &Measurement) -> (i32, i32) {
        (mes.compute(dpi, self.0 as f32) as i32, mes.compute(dpi, self.1 as f32) as i32)
    }

    fn compute_percentage_value(&self, total: (i32, i32), percentage: u8) -> (i32, i32) {
        ((percentage as f32).value(total.0 as f32) as i32, (percentage as f32).value(total.1 as f32) as i32)
    }

    fn compute_percentage_value_self(&self, percentage: u8) -> (i32, i32) {
        ((percentage as f32).value(self.0 as f32) as i32, (percentage as f32).value(self.1 as f32) as i32)
    }
}

macro_rules! impl_gvv_prim {
    ($($typ:ty),*) => {
        $(
            impl GuiValueValue<$typ> for $typ {
                fn compute_measurement(&self, dpi: f32, mes: &Measurement) -> $typ {
                    mes.compute(dpi, *self as f32) as $typ
                }

                fn compute_percentage_value(&self, total: $typ, percentage: u8) -> $typ {
                    (percentage as f32).value(total as f32) as $typ
                }

                fn compute_percentage_value_self(&self, percentage: u8) -> $typ {
                    (percentage as f32).value(*self as f32) as $typ
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

                fn compute_percentage_value_self(&self, percentage: u8) -> $typ {
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
    ParentPercentage(u8),
    Inherit(),
    Clone(&'static GuiStyle)
}

impl<T: Clone + GuiValueValue<T>> GuiValue<T> {
    pub fn unwrap<F>(&self, compute_supply: &GuiValueComputeSupply, mut resolve_supply: F) -> T where F: FnMut(&GuiStyle) -> &GuiValue<T> {
        match self {
            GuiValue::Just(t) => {t.clone()},
            GuiValue::Measurement(t, mes) => {t.compute_measurement(compute_supply.get_dpi(), mes)},
            GuiValue::Percentage(t, perc) => {t.compute_percentage_value(t.clone(), perc.clone())},
            GuiValue::ParentPercentage(p) => {resolve_supply(&compute_supply.get_parent().clone().expect("Set 'ParentPercentage' on element without parent!").borrow().info().style).unwrap(compute_supply, resolve_supply).compute_percentage_value_self(p.clone())}
            GuiValue::Inherit() => {resolve_supply(&compute_supply.get_parent().clone().expect("Set 'Inherit' on element without parent!").borrow().info().style).unwrap(compute_supply, resolve_supply)},
            GuiValue::Clone(other) => {resolve_supply(other).unwrap(compute_supply, resolve_supply)}
        }
    }
}

impl<T: Clone + GuiValueValue<T>> Clone for GuiValue<T> {
    fn clone(&self) -> Self {
        return match self {
            GuiValue::Just(t) => GuiValue::Just(t.clone()),
            GuiValue::Measurement(t, mes) => GuiValue::Measurement(t.clone(), mes.clone()),
            GuiValue::Percentage(t, perc) => GuiValue::Percentage(t.clone(), *perc),
            GuiValue::ParentPercentage(p) => GuiValue::ParentPercentage(*p),
            GuiValue::Inherit() => GuiValue::Inherit(),
            GuiValue::Clone(other) => GuiValue::Clone(other)
        }
    }
}

impl<T: Copy + GuiValueValue<T>> Copy for GuiValue<T> {}

impl<T: Default + Clone + GuiValueValue<T>> Default for GuiValue<T> {
    fn default() -> Self {
        GuiValue::Just(T::default())
    }
}

#[macro_export]
macro_rules! resolve {
    ($info:expr, $prop:ident) => {
        $info.style.$prop.unwrap(&$info.compute_supply, |s| {&s.$prop})
    };
    ($info:ident, $prop:ident) => {
        $info.style.$prop.unwrap(&$info.compute_supply, |s| {&s.$prop})
    };
}