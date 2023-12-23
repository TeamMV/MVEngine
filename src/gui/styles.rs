use crate::gui::elements::{GuiElement, GuiElementImpl};
use mvutils::unsafe_utils::Unsafe;
use std::convert::Infallible;
use std::sync::Arc;

pub struct Style {
    //position
    pub x: GuiValue<i32>,
    pub y: GuiValue<i32>,
    pub width: GuiValue<i32>,
    pub height: GuiValue<i32>,
    pub padding: SideStyle,
    pub margin: SideStyle,
    pub origin: GuiValue<Origin>,
}

pub enum Origin {
    TopLeft,
    BottomLeft,
    TopRight,
    BottomRight,
    Center,
    Custom(i32, i32),
}

impl Origin {
    pub fn is_right(&self) -> bool {
        true
    }
}

pub struct TextStyle {}

pub struct SideStyle {
    pub top: GuiValue<i32>,
    pub bottom: GuiValue<i32>,
    pub left: GuiValue<i32>,
    pub right: GuiValue<i32>,
}

impl SideStyle {
    pub fn all_i32(v: i32) -> Self {
        Self {
            top: GuiValue::Just(v),
            bottom: GuiValue::Just(v),
            left: GuiValue::Just(v),
            right: GuiValue::Just(v),
        }
    }

    pub fn all(v: GuiValue<i32>) -> Self {
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

#[derive(Clone)]
pub enum GuiValue<T: Clone + 'static> {
    None,
    Auto,
    Inherit,
    Clone(Arc<dyn GuiElement>),
    Just(T),
    Measurement(Unit),
}

impl<T: Clone + 'static> GuiValue<T> {
    pub fn resolve<F>(&self, dpi: f32, parent: Option<Arc<dyn GuiElement>>, map: F) -> Option<T>
    where
        F: Fn(&Style) -> &GuiValue<T>,
    {
        match self {
            GuiValue::None => None,
            GuiValue::Auto => None,
            GuiValue::Inherit => map(parent.clone().unwrap_or_else(no_parent).style()).resolve(
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
            GuiValue::Clone(e) => map(e.style()).resolve(dpi, e.parent(), map),
            GuiValue::Just(v) => Some(v.clone()),
            GuiValue::Measurement(u) => {
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
        self != GuiValue::None && self != GuiValue::Auto
    }
}

fn no_parent<T>() -> T {
    panic!("Called Inherit on GuiElement without parent")
}

#[macro_export]
macro_rules! resolve {
    ($elem:ident, $($style:tt)*) => {
        $elem.style().$($style)*.resolve($elem.resolve_context().dpi, $elem.parent(), |s| {&s.$($style)*})
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
    Px(i32),
    MM(f32),
    CM(f32),
    M(f32),
    In(f32),
    Twip(f32),
    Mil(f32),
    Point(f32),
    Pica(f32),
    Foot(f32),
    Yard(f32),
    Link(f32),
    Rod(f32),
    Chain(f32),
    Line(f32),
    BarleyCorn(f32),
    Nail(f32),
    Finger(f32),
    Stick(f32),
    Palm(f32),
    Shaftment(f32),
    Span(f32),
    Quarter(f32),
    Pace(f32),
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
