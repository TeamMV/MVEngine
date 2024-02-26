use crate::render::color::RgbColor;
use crate::render::text::Font;
use crate::resources::resources::R;
use crate::ui::background::{Background, BackgroundInfo, RectangleBackground};
use crate::ui::elements::{UiElement, UiElementImpl};
use mvutils::unsafe_utils::Unsafe;
use num_traits::Num;
use std::convert::Infallible;
use std::sync::Arc;
use winit::event::VirtualKeyCode::O;

pub struct Style {
    //position
    pub x: UiValue<i32>,
    pub y: UiValue<i32>,
    pub width: UiValue<i32>,
    pub height: UiValue<i32>,
    pub padding: SideStyle,
    pub margin: SideStyle,
    pub origin: UiValue<Origin>,
    pub position: UiValue<Position>,
    pub rotation_origin: UiValue<Origin>,
    pub rotation: UiValue<f32>,

    pub text: TextStyle,

    pub background: BackgroundInfo,
}

#[derive(Default, Clone, Copy, Eq, PartialEq)]
pub enum Origin {
    TopLeft,
    #[default]
    BottomLeft,
    TopRight,
    BottomRight,
    Center,
    Custom(i32, i32),
}

impl Origin {
    pub fn is_right(&self) -> bool {
        matches!(self, Origin::BottomRight | Origin::TopRight)
    }

    pub fn is_left(&self) -> bool {
        *self == Origin::BottomLeft || *self == Origin::TopLeft
    }

    pub fn get_custom(&self) -> Option<(i32, i32)> {
        if let Origin::Custom(x, y) = self {
            Some((*x, *y))
        } else {
            None
        }
    }

    pub fn resolve<E>(&self, elem: &E) -> Point<i32>
    where
        E: UiElement + ?Sized,
    {
        match self {
            Origin::TopLeft => Point::new(elem.border_x(), elem.border_y() + elem.height()),
            Origin::BottomLeft => Point::new(elem.border_x(), elem.border_y()),
            Origin::TopRight => Point::new(
                elem.border_x() + elem.width(),
                elem.border_y() + elem.height(),
            ),
            Origin::BottomRight => Point::new(elem.border_x() + elem.width(), elem.border_y()),
            Origin::Center => Point::new(
                elem.border_x() + elem.width() / 2,
                elem.border_y() + elem.height() / 2,
            ),
            Origin::Custom(x, y) => Point::new(*x, *y),
        }
    }
}

#[derive(Default, Clone, Copy, Eq, PartialEq)]
pub enum Position {
    Absolute,
    #[default]
    Relative,
}

pub struct TextStyle {
    pub size: UiValue<f32>,
    pub kerning: UiValue<f32>,
    pub skew: UiValue<f32>,
    pub stretch: UiValue<Dimension<f32>>,
    pub font: UiValue<Arc<Font>>,
}

impl TextStyle {
    pub const fn initial() -> Self {
        Self {
            size: UiValue::Measurement(Unit::BarleyCorn(1.0)),
            kerning: UiValue::None,
            skew: UiValue::None,
            stretch: UiValue::None,
            font: UiValue::Auto,
        }
    }
}

#[derive(Clone)]
pub struct Dimension<T: Num + Clone> {
    pub width: T,
    pub height: T,
}

impl<T: Num + Clone> Dimension<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

#[derive(Clone)]
pub struct Point<T: Num + Clone> {
    pub x: T,
    pub y: T,
}

impl<T: Num + Clone> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

pub struct SideStyle {
    pub top: UiValue<i32>,
    pub bottom: UiValue<i32>,
    pub left: UiValue<i32>,
    pub right: UiValue<i32>,
}

impl SideStyle {
    pub const fn all_i32(v: i32) -> Self {
        Self {
            top: UiValue::Just(v),
            bottom: UiValue::Just(v),
            left: UiValue::Just(v),
            right: UiValue::Just(v),
        }
    }

    pub fn all(v: UiValue<i32>) -> Self {
        Self {
            top: v.clone(),
            bottom: v.clone(),
            left: v.clone(),
            right: v,
        }
    }

    pub fn horizontal(&self) -> i32 {
        unimplemented!()
    }

    pub fn vertical(&self) -> i32 {
        unimplemented!()
    }
}

#[derive(Clone, Default)]
pub enum UiValue<T: Clone + 'static> {
    #[default]
    None,
    Auto,
    Inherit,
    Clone(Arc<dyn UiElement>),
    Just(T),
    Measurement(Unit),
}

impl<T: Clone + 'static> UiValue<T> {
    pub fn resolve<F>(&self, dpi: f32, parent: Option<Arc<dyn UiElement>>, map: F) -> Option<T>
    where
        F: Fn(&Style) -> &UiValue<T>,
    {
        match self {
            UiValue::None => None,
            UiValue::Auto => None,
            UiValue::Inherit => map(parent.clone().unwrap_or_else(no_parent).style()).resolve(
                dpi,
                Some(
                    parent
                        .clone()
                        .unwrap_or_else(no_parent)
                        .parent()
                        .unwrap_or_else(no_parent),
                ),
                map,
            ),
            UiValue::Clone(e) => map(e.style()).resolve(dpi, e.parent(), map),
            UiValue::Just(v) => Some(v.clone()),
            UiValue::Measurement(u) => {
                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<i32>() {
                    unsafe {
                        let a = u.as_px(dpi);
                        Some(Unsafe::cast_ref::<i32, T>(&a).clone())
                    }
                } else {
                    None
                }
            }
        }
    }

    pub fn is_set(&self) -> bool {
        !matches!(self, UiValue::None | UiValue::Auto)
    }
}

//impl<T: Clone + 'static> PartialEq for UiValue<T> {
//    fn eq(&self, other: &Self) -> bool {
//        matches!(self, other)
//    }
//}

fn no_parent<T>() -> T {
    panic!("Called Inherit on UiElement without parent")
}

impl Default for Style {
    fn default() -> Self {
        Self {
            x: UiValue::Just(0),
            y: UiValue::Just(0),
            width: UiValue::Auto,
            height: UiValue::Auto,
            padding: SideStyle::all_i32(0),
            margin: SideStyle::all_i32(0),
            origin: UiValue::Just(Origin::BottomLeft),
            position: UiValue::Just(Position::Relative),
            rotation_origin: UiValue::Just(Origin::Center),
            rotation: UiValue::Just(0.0),
            text: TextStyle::initial(),
            background: BackgroundInfo::default(),
        }
    }
}

#[macro_export]
macro_rules! resolve {
    ($elem:ident, $($style:ident).*) => {
        {
            let s: &UiValue<_> = &$elem.style().$($style).*;
            let v: Option<_> = s.resolve($elem.resolve_context().dpi, $elem.parent(), |s| {&s.$($style).*});
            if let Some(v) = v {
                v
            }
            else {
                log::error!("UiValue {} failed to resolve on element {}", stringify!($($style).*), $elem.id());
                $crate::ui::styles::Style::default().$($style).*
                .resolve($elem.resolve_context().dpi, None, |s| {&s.$($style).*})
                .expect("Default style could not be resolved")
            }
        }
    };
}

pub(crate) struct ResCon {
    pub dpi: f32,
}

impl ResCon {
    pub(crate) fn set_dpi(&mut self, dpi: f32) {
        self.dpi = dpi;
    }
}

#[derive(Clone, Copy)]
pub enum Unit {
    Px(i32),         // px
    MM(f32),         // mm
    CM(f32),         // cm
    M(f32),          // m
    In(f32),         // in
    Twip(f32),       // twip
    Mil(f32),        // mil
    Point(f32),      // pt
    Pica(f32),       // pica
    Foot(f32),       // ft
    Yard(f32),       // yd
    Link(f32),       // lk
    Rod(f32),        // rd
    Chain(f32),      // ch
    Line(f32),       // ln
    BarleyCorn(f32), // bc
    Nail(f32),       // nl
    Finger(f32),     // fg
    Stick(f32),      // sk
    Palm(f32),       // pm
    Shaftment(f32),  // sf
    Span(f32),       // sp
    Quarter(f32),    // qr
    Pace(f32),       // pc
}

impl Unit {
    pub fn as_px(&self, dpi: f32) -> i32 {
        match self {
            Unit::Px(px) => *px,
            Unit::MM(value) => ((value / 25.4) * dpi) as i32,
            Unit::CM(value) => ((value / 2.54) * dpi) as i32,
            Unit::M(value) => (value * dpi) as i32,
            Unit::In(value) => (value * dpi) as i32,
            Unit::Twip(value) => ((value / 1440.0) * dpi) as i32,
            Unit::Mil(value) => ((value / 1000.0) * dpi) as i32,
            Unit::Point(value) => (value * (dpi / 72.0)) as i32,
            Unit::Pica(value) => (value * (dpi / 6.0)) as i32,
            Unit::Foot(value) => ((value * 12.0) * dpi) as i32,
            Unit::Yard(value) => ((value * 36.0) * dpi) as i32,
            Unit::Link(value) => ((value * 7.92) * dpi) as i32,
            Unit::Rod(value) => ((value * 198.0) * dpi) as i32,
            Unit::Chain(value) => ((value * 792.0) * dpi) as i32,
            Unit::Line(value) => ((value * 0.792) * dpi) as i32,
            Unit::BarleyCorn(value) => ((value * 0.125) * dpi) as i32,
            Unit::Nail(value) => ((value * 0.25) * dpi) as i32,
            Unit::Finger(value) => ((value * 0.375) * dpi) as i32,
            Unit::Stick(value) => ((value * 0.5) * dpi) as i32,
            Unit::Palm(value) => ((value * 3.0) * dpi) as i32,
            Unit::Shaftment(value) => ((value * 6.0) * dpi) as i32,
            Unit::Span(value) => ((value * 9.0) * dpi) as i32,
            Unit::Quarter(value) => ((value * 36.0) * dpi) as i32,
            Unit::Pace(value) => ((value * 30.0) * dpi) as i32,
        }
    }
}
