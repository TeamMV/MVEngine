use crate::gui::elements::{GuiElement, GuiElementImpl};
use std::sync::Arc;

pub struct Style {}

pub enum GuiValue<T: Clone> {
    None,
    Auto,
    Inherit,
    Just(T),
    Measurement(Unit),
}

impl<T: Clone> GuiValue<T> {
    fn resolve<F>(&self, dpi: f32, parent_elem: GuiElementImpl) -> Option<T>
    where
        F: Fn(&Style) -> &GuiValue<T>,
    {
        match self {
            GuiValue::None => None,
            GuiValue::Auto => None,
            GuiValue::Inherit => {}
            GuiValue::Just(v) => Some(v.clone()),
            GuiValue::Measurement(u) => Some(Arc::new(u.as_px(dpi))),
        }
    }
}

struct ResCon(f32);

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
