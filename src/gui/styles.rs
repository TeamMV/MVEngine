use std::sync::Arc;

use mvutils::screen::Measurement;
use mvutils::utils::{Percentage, TetrahedronOp};
use num_traits::Num;
use mvcore_proc_macro::style_interpolator;

use crate::gui::components::{GuiElement, GuiElementInfo};
use crate::gui::ease::{Easing, FromF32, IntoF32};
use crate::render::color::{Color, Gradient, RGB, RgbColor, RgbGradient};
use crate::render::draw2d::CanvasStyle;
use crate::render::text::TypeFace;

pub struct GuiStyle {
    //common
    pub background_color: GuiValue<Gradient<RGB, f32>>,
    pub foreground_color: GuiValue<Gradient<RGB, f32>>,
    pub text_color: GuiValue<Gradient<RGB, f32>>,
    pub text_chroma: GuiValue<bool>,
    pub text_chroma_compress: GuiValue<f32>,
    pub text_chroma_tilt: GuiValue<f32>,
    pub text_size: GuiValue<i32>,
    pub font: GuiValue<Option<Arc<TypeFace>>>,
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
    pub position: GuiValue<Positioning>,
    pub width: GuiValue<i32>,
    pub height: GuiValue<i32>,
    pub x: GuiValue<i32>,
    pub y: GuiValue<i32>,
    pub origin: GuiValue<Origin>,
    pub rotation: GuiValue<f32>,
    pub rotation_center: GuiValue<RotationCenter>,
    pub z_index: GuiValue<u16>,
    //Layout
    pub vertical_align: GuiValue<VerticalAlign>,
    pub horizontal_align: GuiValue<HorizontalAlign>,
    pub vertical_item_align: GuiValue<VerticalAlign>,
    pub horizontal_item_align: GuiValue<HorizontalAlign>,
    pub scroll: GuiValue<Scroll>,
    pub overflow: GuiValue<Overflow>,
    pub size: GuiValue<Size>,
    pub spacing: GuiValue<i32>,
    pub item_direction: GuiValue<Direction>,
    //scrollbar
    pub scrollbar_slider_style: GuiValue<ScrollbarSlider>,
    pub scrollbar_vertical_align: GuiValue<VerticalAlign>,
    pub scrollbar_horizontal_align: GuiValue<HorizontalAlign>,
    pub scrollbar_track_color: GuiValue<Gradient<RGB, f32>>,
    pub scrollbar_slider_color: GuiValue<Gradient<RGB, f32>>,
    pub scrollbar_track_border_width: GuiValue<i32>,
    pub scrollbar_track_border_radius: GuiValue<i32>,
    pub scrollbar_track_border_style: GuiValue<BorderStyle>,
    pub scrollbar_track_border_color: GuiValue<Gradient<RGB, f32>>,
    pub scrollbar_slider_border_width: GuiValue<i32>,
    pub scrollbar_slider_border_radius: GuiValue<i32>,
    pub scrollbar_slider_border_style: GuiValue<BorderStyle>,
    pub scrollbar_slider_border_color: GuiValue<Gradient<RGB, f32>>,
    pub scrollbar_width: GuiValue<i32>,
    pub scrollbar_mode: GuiValue<ScrollbarMode>,
    pub scrollbar_leave_easing_func: GuiValue<Easing>,
    pub scroll_easing_func: GuiValue<Easing>,
}

#[macro_export]
macro_rules! value {
    ($val:tt px) => {
        GuiValue::Measurement($val, Measurement::PX)
    };
    ($val:tt mm) => {
        GuiValue::Measurement($val, Measurement::MM)
    };
    ($val:tt cm) => {
        GuiValue::Measurement($val, Measurement::CM)
    };
    ($val:tt dm) => {
        GuiValue::Measurement($val, Measurement::DM)
    };
    ($val:tt m) => {
        GuiValue::Measurement($val, Measurement::CM)
    };
    ($val:tt IN) => {
        GuiValue::Measurement($val, Measurement::IN)
    };
    ($val:tt inch) => {
        GuiValue::Measurement($val, Measurement::IN)
    };
    ($val:tt inches) => {
        GuiValue::Measurement($val, Measurement::IN)
    };
    ($val:tt ft) => {
        GuiValue::Measurement($val, Measurement::FT)
    };
    ($val:tt p) => {
        GuiValue::ParentPercentage($val)
    };
    ($val:tt percent) => {
        GuiValue::ParentPercentage($val)
    };
    ($val:tt clone) => {
        GuiValue::Clone($val)
    };
    (inherit) => {
        GuiValue::Inherit()
    };
    ($val:expr) => {
        GuiValue::Just($val)
    };
}

#[macro_export]
macro_rules! setup {
    (
        $s:expr => {
            $($key:ident$(: $value:tt $($suffix:ident)*)?),*
        }
    ) => {
        $(
            $s.$key$( = $crate::value!($value $($suffix)*))?;
        )*
    }
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
            position: Default::default(),
            width: Default::default(),
            height: Default::default(),
            x: Default::default(),
            y: Default::default(),
            origin: Default::default(),
            rotation: Default::default(),
            rotation_center: Default::default(),
            z_index: Default::default(),
            vertical_align: Default::default(),
            horizontal_align: Default::default(),
            vertical_item_align: Default::default(),
            horizontal_item_align: Default::default(),
            scroll: Default::default(),
            overflow: Default::default(),
            size: Default::default(),
            spacing: GuiValue::Just(5),
            item_direction: Default::default(),
            scrollbar_slider_style: Default::default(),
            scrollbar_vertical_align: Default::default(),
            scrollbar_horizontal_align: Default::default(),
            scrollbar_track_color: Default::default(),
            scrollbar_slider_color: Default::default(),
            scrollbar_track_border_width: Default::default(),
            scrollbar_track_border_radius: Default::default(),
            scrollbar_track_border_style: Default::default(),
            scrollbar_track_border_color: Default::default(),
            scrollbar_slider_border_width: Default::default(),
            scrollbar_slider_border_radius: Default::default(),
            scrollbar_slider_border_style: Default::default(),
            scrollbar_slider_border_color: Default::default(),
            scrollbar_width: Default::default(),
            scrollbar_mode: Default::default(),
            scrollbar_leave_easing_func: Default::default(),
            scroll_easing_func: Default::default(),
        }
    }
}

#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub enum ViewState {
    #[default]
    Visible,
    Invisible,
    Gone,
}

#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub enum Positioning {
    #[default]
    Relative,
    Absolute,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Default)]
pub enum BorderStyle {
    #[default]
    Square,
    Round,
    Triangle,
}

impl BorderStyle {
    pub fn as_cnvs_style(&self) -> CanvasStyle {
        return match self {
            BorderStyle::Square => CanvasStyle::Square,
            BorderStyle::Round => CanvasStyle::Round,
            BorderStyle::Triangle => CanvasStyle::Triangle,
        };
    }
}

#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub enum Origin {
    #[default]
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}

impl Origin {
    pub fn is_top(&self) -> bool {
        self == &Origin::TopLeft || self == &Origin::TopRight
    }

    pub fn is_bottom(&self) -> bool {
        self == &Origin::BottomLeft || self == &Origin::BottomRight
    }

    pub fn is_right(&self) -> bool {
        self == &Origin::TopRight || self == &Origin::BottomRight
    }

    pub fn is_left(&self) -> bool {
        self == &Origin::TopLeft || self == &Origin::BottomLeft
    }
}

#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub enum VerticalAlign {
    #[default]
    Top,
    Bottom,
    Center,
}

#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub enum HorizontalAlign {
    #[default]
    Left,
    Right,
    Center,
}

#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub enum Scroll {
    #[default]
    Vertical,
    Horizontal,
    Both,
    None,
}

#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub enum Overflow {
    #[default]
    Cut,
    Clamp,
    Ignore,
}

#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub enum Size {
    #[default]
    Content,
    Fixed,
}

#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub enum ScrollbarMode {
    #[default]
    Stay,
    FadeOnLeave,
    HideOnLeave,
}

#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub enum ScrollbarSlider {
    #[default]
    Slider,
    Thumb,
}

#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub enum Direction {
    #[default]
    LeftRight,
    UpDown,
}

#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub enum RotationCenter {
    #[default]
    Element,
    Custom((i32, i32)),
}

pub trait GuiValueValue<T> {
    fn compute_measurement(&self, dpi: f32, mes: &Measurement) -> T;
    fn compute_percentage_value(&self, total: T, percentage: u8) -> T;
    fn compute_percentage_value_self(&self, percentage: u8) -> T;
}

impl GuiValueValue<(i32, i32)> for (i32, i32) {
    fn compute_measurement(&self, dpi: f32, mes: &Measurement) -> (i32, i32) {
        (
            mes.compute(dpi, self.0 as f32) as i32,
            mes.compute(dpi, self.1 as f32) as i32,
        )
    }

    fn compute_percentage_value(&self, total: (i32, i32), percentage: u8) -> (i32, i32) {
        (
            (percentage as f32).value(total.0 as f32) as i32,
            (percentage as f32).value(total.1 as f32) as i32,
        )
    }

    fn compute_percentage_value_self(&self, percentage: u8) -> (i32, i32) {
        (
            (percentage as f32).value(self.0 as f32) as i32,
            (percentage as f32).value(self.1 as f32) as i32,
        )
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

impl_unreachable_gvv!(
    Gradient<RGB, f32>,
    ViewState,
    Positioning,
    BorderStyle,
    Origin,
    Option<Arc<TypeFace>>,
    bool,
    VerticalAlign,
    HorizontalAlign,
    Scroll,
    ScrollbarMode,
    ScrollbarSlider,
    Overflow,
    Size,
    Direction,
    Easing,
    RotationCenter);

pub struct GuiValueComputeSupply {
    pub dpi: f32,
    pub parent: Option<Arc<GuiElement>>,
}

impl GuiValueComputeSupply {
    pub fn new(dpi: f32, parent: Option<Arc<GuiElement>>) -> Self {
        GuiValueComputeSupply { dpi, parent }
    }

    fn get_dpi(&self) -> f32 {
        self.dpi
    }

    fn get_parent(&self) -> &Option<Arc<GuiElement>> {
        &self.parent
    }
}

pub enum GuiValue<T: Clone + GuiValueValue<T>> {
    Just(T),
    Measurement(T, Measurement),
    Percentage(T, u8),
    ParentPercentage(u8),
    Inherit(),
    Clone(&'static GuiStyle),
}

impl<T: Clone + GuiValueValue<T>> GuiValue<T> {
    pub fn unwrap<F>(&self, compute_supply: &GuiValueComputeSupply, mut resolve_supply: F) -> T
    where
        F: FnMut(&GuiStyle) -> &GuiValue<T>,
    {
        match self {
            GuiValue::Just(t) => t.clone(),
            GuiValue::Measurement(t, mes) => t.compute_measurement(compute_supply.get_dpi(), mes),
            GuiValue::Percentage(t, perc) => t.compute_percentage_value(t.clone(), perc.clone()),
            GuiValue::ParentPercentage(p) => resolve_supply(
                &compute_supply
                    .get_parent()
                    .clone()
                    .expect("Set 'ParentPercentage' on element without parent!")
                    .info()
                    .style,
            )
            .unwrap(compute_supply, resolve_supply)
            .compute_percentage_value_self(p.clone()),
            GuiValue::Inherit() => resolve_supply(
                &compute_supply
                    .get_parent()
                    .clone()
                    .expect("Set 'Inherit' on element without parent!")
                    .info()
                    .style,
            )
            .unwrap(compute_supply, resolve_supply),
            GuiValue::Clone(other) => resolve_supply(other).unwrap(compute_supply, resolve_supply),
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
            GuiValue::Clone(other) => GuiValue::Clone(other),
        };
    }
}

#[macro_export]
macro_rules! resolve {
    ($info:expr, $prop:ident) => {
        $info
            .style
            .$prop
            .unwrap(&$info.compute_supply, |s| &s.$prop)
    };
    ($info:ident, $prop:ident) => {
        $info
            .style
            .$prop
            .unwrap(&$info.compute_supply, |s| &s.$prop)
    };
}

impl<T: Copy + GuiValueValue<T>> Copy for GuiValue<T> {}

impl<T: Default + Clone + GuiValueValue<T>> Default for GuiValue<T> {
    fn default() -> Self {
        GuiValue::Just(T::default())
    }
}

pub trait Interpolator<T> {
    fn interpolate(&self, t: T, start: T, end: T, progress: f32, easing: Easing) -> T {
        (progress < 50.0).yn(start, end)
    }
}

impl<T: Num + Copy + FromF32 + IntoF32> Interpolator<T> for T {
    fn interpolate(&self, t: T, start: T, end: T, progress: f32, mut easing: Easing) -> T {
        easing.xr = 0.0..100.0;
        easing.yr = start.into_f32()..end.into_f32();
        T::from_f32(easing.get(progress))
    }
}

macro_rules! interpolator_basic {
    ($($typ:ty,)*) => {
        $(
            impl Interpolator<$typ> for $typ {}
        )*
    };
}

interpolator_basic!(
    RgbColor,
    RgbGradient,
    ViewState,
    Positioning,
    BorderStyle,
    Origin,
    VerticalAlign,
    HorizontalAlign,
    Scroll,
    ScrollbarMode,
    ScrollbarSlider,
    Overflow,
    Size,
    Direction,
    Easing,
    RotationCenter,
);

#[macro_export]
macro_rules! style {
    ($elem:ident, $field:ident = $value:expr) => {
        $elem.info_mut().style.$field = $value
    };
}