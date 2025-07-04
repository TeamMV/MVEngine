pub mod enums;
pub mod groups;
pub mod interpolate;
pub mod types;
pub mod unit;

use crate::blanked_partial_ord;
use crate::color::RgbColor;
use crate::ui::elements::components::scroll;
use crate::ui::elements::{UiElement, UiElementState, UiElementStub};
use crate::ui::res::MVR;
use crate::ui::styles::enums::Overflow;
use crate::ui::styles::groups::{ScrollBarStyle, VectorField};
use crate::ui::styles::interpolate::BasicInterpolatable;
use crate::ui::styles::types::Dimension;
use crate::window::Window;
use enums::{BackgroundRes, ChildAlign, Direction, Geometry, Origin, Position, TextAlign, TextFit};
use groups::{LayoutField, ShapeStyle, SideStyle, TextStyle, TransformStyle};
use interpolate::Interpolator;
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use mvutils::{enum_val_ref, lazy};
use std::any::TypeId;
use std::rc::Rc;
use std::str::FromStr;
use mvutils::save::{Loader, Savable, Saver};
use unit::Unit;

lazy! {
    pub static DEFAULT_STYLE: UiStyle = UiStyle {
        x: UiValue::Just(0).to_field().to_resolve(),
        y: UiValue::Just(0).to_field().to_resolve(),
        width: UiValue::Auto.to_field().to_resolve(),
        height: UiValue::Auto.to_field().to_resolve(),
        padding: SideStyle::all(UiValue::Measurement(Unit::BeardFortnight(0.5)).to_field().to_resolve()),
        margin: SideStyle::all(UiValue::Measurement(Unit::BeardFortnight(0.5)).to_field().to_resolve()),
        origin: UiValue::Just(Origin::BottomLeft).to_resolve(),
        position: UiValue::Just(Position::Relative).to_resolve(),
        direction: UiValue::Just(Direction::Horizontal).to_resolve(),
        child_align_x: UiValue::Just(ChildAlign::Start).to_resolve(),
        child_align_y: UiValue::Just(ChildAlign::Start).to_resolve(),
        background: ShapeStyle {
            resource: UiValue::Just(BasicInterpolatable::new(BackgroundRes::Color)).to_resolve(),
            color: UiValue::Just(RgbColor::white()).to_resolve(),
            texture: UiValue::None.to_resolve(),
            shape: UiValue::Just(BasicInterpolatable::new(Geometry::Shape(MVR.shape.rect))).to_resolve(),
        },
        border: ShapeStyle {
            resource: UiValue::Just(BasicInterpolatable::new(BackgroundRes::Color)).to_resolve(),
            color: UiValue::Just(RgbColor::black()).to_resolve(),
            texture: UiValue::None.to_resolve(),
            shape: UiValue::Just(BasicInterpolatable::new(Geometry::Adaptive(MVR.adaptive.void_rect))).to_resolve(),
            //shape: UiValue::Just(BasicInterpolatable::new(UiShape::Shape(MVR.shape.rect))).to_resolve(),
        },
        text: TextStyle {
            size: UiValue::Measurement(Unit::BeardFortnight(1.0)).to_field().to_resolve(),
            kerning: UiValue::Just(0.0).to_field().to_resolve(),
            skew: UiValue::Just(0.0).to_field().to_resolve(),
            stretch: UiValue::Just(Dimension::new(1.0, 1.0)).to_field().to_resolve(),
            font: UiValue::Just(MVR.font.default).to_field().to_resolve(),
            fit: UiValue::Just(TextFit::ExpandParent).to_resolve(),
            color: UiValue::Just(RgbColor::black()).to_resolve(),
            align_x: UiValue::Just(TextAlign::Middle).to_resolve(),
            align_y: UiValue::Just(TextAlign::Middle).to_resolve(),
        },
        transform: TransformStyle {
            translate: VectorField::splat(UiValue::Just(0).to_field().to_resolve()),
            scale: VectorField::splat(UiValue::Just(1.0).to_field().to_resolve()),
            rotate: UiValue::Just(0.0).to_field().to_resolve(),
            origin: UiValue::Just(Origin::BottomLeft).to_field().to_resolve(),
        },
        overflow_x: UiValue::Just(Overflow::Normal).to_resolve(),
        overflow_y: UiValue::Just(Overflow::Normal).to_resolve(),
        scrollbar: ScrollBarStyle {
            track: ShapeStyle {
                resource: UiValue::Just(BasicInterpolatable::new(BackgroundRes::Color)).to_resolve(),
                color: UiValue::Just(scroll::OUTER_COLOR.clone()).to_resolve(),
                texture: UiValue::None.to_resolve(),
                shape: UiValue::Just(BasicInterpolatable::new(Geometry::Shape(MVR.shape.rect))).to_resolve(),
            },
            knob: ShapeStyle {
                resource: UiValue::Just(BasicInterpolatable::new(BackgroundRes::Color)).to_resolve(),
                color: UiValue::Just(scroll::INNER_COLOR.clone()).to_resolve(),
                texture: UiValue::None.to_resolve(),
                shape: UiValue::Just(BasicInterpolatable::new(Geometry::Shape(MVR.shape.rect))).to_resolve(),
            },
            size: UiValue::Measurement(Unit::BeardFortnight(1.0)).to_resolve(),
        }
    };

    pub static EMPTY_STYLE: UiStyle = UiStyle {
        x: UiValue::Unset.to_field().to_resolve(),
        y: UiValue::Unset.to_field().to_resolve(),
        width: UiValue::Unset.to_field().to_resolve(),
        height: UiValue::Unset.to_field().to_resolve(),
        padding: SideStyle::all(UiValue::Unset.to_field().to_resolve()),
        margin: SideStyle::all(UiValue::Unset.to_field().to_resolve()),
        origin: UiValue::Unset.to_resolve(),
        position: UiValue::Unset.to_resolve(),
        direction: UiValue::Unset.to_resolve(),
        child_align_x: UiValue::Unset.to_resolve(),
        child_align_y: UiValue::Unset.to_resolve(),
        background: ShapeStyle {
            resource: UiValue::Unset.to_resolve(),
            color: UiValue::Unset.to_resolve(),
            texture: UiValue::Unset.to_resolve(),
            shape: UiValue::Unset.to_resolve(),
        },
        border: ShapeStyle {
            resource: UiValue::Unset.to_resolve(),
            color: UiValue::Unset.to_resolve(),
            texture: UiValue::Unset.to_resolve(),
            shape: UiValue::Unset.to_resolve(),
        },
        text: TextStyle {
            size: UiValue::Unset.to_field().to_resolve(),
            kerning: UiValue::Unset.to_field().to_resolve(),
            skew: UiValue::Unset.to_field().to_resolve(),
            stretch: UiValue::Unset.to_field().to_resolve(),
            font: UiValue::Unset.to_field().to_resolve(),
            fit: UiValue::Unset.to_field().to_resolve(),
            color: UiValue::Unset.to_resolve(),
            align_x: UiValue::Unset.to_resolve(),
            align_y: UiValue::Unset.to_resolve(),
        },
        transform: TransformStyle {
            translate: VectorField::splat(UiValue::Unset.to_field().to_resolve()),
            scale: VectorField::splat(UiValue::Unset.to_field().to_resolve()),
            rotate: UiValue::Unset.to_field().to_resolve(),
            origin: UiValue::Unset.to_field().to_resolve(),
        },
        overflow_x: UiValue::Unset.to_resolve(),
        overflow_y: UiValue::Unset.to_resolve(),
        scrollbar: ScrollBarStyle {
            track: ShapeStyle {
                resource: UiValue::Unset.to_resolve(),
                color: UiValue::Unset.to_resolve(),
                texture: UiValue::Unset.to_resolve(),
                shape: UiValue::Unset.to_resolve(),
            },
            knob: ShapeStyle {
                resource: UiValue::Unset.to_resolve(),
                color: UiValue::Unset.to_resolve(),
                texture: UiValue::Unset.to_resolve(),
                shape: UiValue::Unset.to_resolve(),
            },
            size: UiValue::Measurement(Unit::BeardFortnight(1.0)).to_resolve(),
        }
    };
}

#[macro_export]
macro_rules! resolve {
    ($elem:ident, $($style:ident).*) => {
        {
            let s = &$elem.style().$($style).*;
            let v: ResolveResult<_> = s.resolve($elem.state().ctx.dpi, $elem.state().parent.clone(), |s| {&s.$($style).*});
            if v.is_use_default() {
                $crate::ui::styles::DEFAULT_STYLE.$($style).*
                .resolve($elem.state().ctx.dpi, None, |s| {&s.$($style).*})
            } else {
                v
            }
        }
    };
}

#[macro_export]
macro_rules! modify_style {
    ($($style:ident).* = $($ac:tt)*) => {
        $($style).*.for_value(|v| *v = $($ac)*);
    };
    ($($style:ident).*! = $($ac:tt)*) => {
        $($style).*.x.for_value(|v| *v = $($ac)*);
        $($style).*.y.for_value(|v| *v = $($ac)*);
    };
    ($($style:ident).*:$acc:ident = $($ac:tt)*) => {
        $($style).*.for_field(|l| (*l).$acc = $($ac)*);
    };
    ($($style:ident).*!:$acc:ident = $($ac:tt)*) => {
        $($style).*.x.for_field(|l| (*l).$acc = $($ac)*);
        $($style).*.y.for_field(|l| (*l).$acc = $($ac)*);
    };
}

pub trait Parseable: Sized {
    fn parse(s: &str) -> Result<Self, String>;
}

impl<T: FromStr> Parseable for T {
    fn parse(s: &str) -> Result<Self, String> {
        T::from_str(s).map_err(|_| format!("{s} cannot be parsed in this context!"))
    }
}

#[derive(Clone)]
pub struct UiStyle {
    pub x: Resolve<i32>,
    pub y: Resolve<i32>,
    pub width: Resolve<i32>,
    pub height: Resolve<i32>,
    pub padding: SideStyle,
    pub margin: SideStyle,
    pub origin: Resolve<Origin>,
    pub position: Resolve<Position>,
    pub direction: Resolve<Direction>,
    pub child_align_x: Resolve<ChildAlign>,
    pub child_align_y: Resolve<ChildAlign>,

    pub background: ShapeStyle,
    pub border: ShapeStyle,

    pub text: TextStyle,
    pub transform: TransformStyle,

    pub overflow_x: Resolve<Overflow>,
    pub overflow_y: Resolve<Overflow>,
    pub scrollbar: ScrollBarStyle,
}

unsafe impl Sync for UiStyle {}
unsafe impl Send for UiStyle {}

impl UiStyle {
    pub fn merge_unset(&mut self, other: &UiStyle) {
        self.x.merge_unset(&other.x);
        self.y.merge_unset(&other.y);
        self.width.merge_unset(&other.width);
        self.height.merge_unset(&other.height);
        self.padding.merge_unset(&other.padding);
        self.margin.merge_unset(&other.margin);
        self.origin.merge_unset(&other.origin);
        self.position.merge_unset(&other.position);
        self.direction.merge_unset(&other.direction);
        self.child_align_x.merge_unset(&other.child_align_x);
        self.child_align_y.merge_unset(&other.child_align_y);
        self.background.merge_unset(&other.background);
        self.border.merge_unset(&other.border);
        self.text.merge_unset(&other.text);
        self.transform.merge_unset(&other.transform);
        self.overflow_x.merge_unset(&other.overflow_x);
        self.overflow_y.merge_unset(&other.overflow_y);
        self.scrollbar.merge_unset(&other.scrollbar);
    }

    pub fn merge_at_set_of(&mut self, to: &UiStyle) {
        self.x.merge_at_set(&to.x);
        self.y.merge_at_set(&to.y);
        self.width.merge_at_set(&to.width);
        self.height.merge_at_set(&to.height);
        self.padding.merge_at_set(&to.padding);
        self.margin.merge_at_set(&to.margin);
        self.origin.merge_at_set(&to.origin);
        self.position.merge_at_set(&to.position);
        self.direction.merge_at_set(&to.direction);
        self.child_align_x.merge_at_set(&to.child_align_x);
        self.child_align_y.merge_at_set(&to.child_align_y);
        self.background.merge_at_set(&to.background);
        self.border.merge_at_set(&to.border);
        self.text.merge_at_set(&to.text);
        self.transform.merge_at_set(&to.transform);
        self.overflow_x.merge_at_set(&to.overflow_x);
        self.overflow_y.merge_at_set(&to.overflow_y);
        self.scrollbar.merge_at_set(&to.scrollbar);
    }
}

blanked_partial_ord!(UiStyle);

blanked_partial_ord!(TextStyle);

blanked_partial_ord!(SideStyle);

#[derive(Clone)]
pub enum Resolve<T: PartialOrd + Clone + 'static> {
    UiValue(UiValue<T>),
    LayoutField(LayoutField<T>),
}

//This guys proc macro is not fancy enough to support generics 🤣🤣🤣🤣🤣🤣🤣🤣🤣🤣🤣🤣🤣🤣🤣🤣
impl<T: Savable + PartialOrd + Clone + 'static> Savable for Resolve<T> {
    fn save(&self, saver: &mut impl Saver) {
        match self {
            Resolve::UiValue(v) => {
                0u8.save(saver);
                v.save(saver);
            }
            Resolve::LayoutField(l) => {
                1u8.save(saver);
                l.value.save(saver);
                l.min.save(saver);
                l.max.save(saver);
            }
        }
    }

    fn load(loader: &mut impl Loader) -> Result<Self, String> {
        let id = u8::load(loader)?;
        match id {
            0 => {
                let v = UiValue::<T>::load(loader)?;
                Ok(Self::UiValue(v))
            },
            1 => {
                let val = UiValue::<T>::load(loader)?;
                let min = UiValue::<T>::load(loader)?;
                let max = UiValue::<T>::load(loader)?;
                let f = LayoutField {
                    value: val,
                    min,
                    max
                };
                Ok(Self::LayoutField(f))
            },
            _ => Err("Illegal id for Resolve when loading!".to_string())
        }
    }
}

impl<T: PartialOrd + Clone + 'static> Resolve<T> {
    pub fn resolve<F>(
        &self,
        dpi: f32,
        parent: Option<Rc<DangerousCell<UiElement>>>,
        map: F,
    ) -> ResolveResult<T>
    where
        F: Fn(&UiStyle) -> &Self,
    {
        match self {
            Resolve::UiValue(v) => v.resolve(dpi, parent, |s| map(s).get_value()),
            Resolve::LayoutField(l) => l.resolve(dpi, parent, |s| map(s).get_field()),
        }
    }

    pub fn get_value(&self) -> &UiValue<T> {
        enum_val_ref!(Resolve, self, UiValue)
    }

    pub fn get_field(&self) -> &LayoutField<T> {
        enum_val_ref!(Resolve, self, LayoutField)
    }

    pub fn for_value<F>(&mut self, f: F)
    where
        F: Fn(&mut UiValue<T>),
    {
        match self {
            Resolve::UiValue(v) => f(v),
            Resolve::LayoutField(l) => f(&mut l.value),
        }
    }

    pub fn for_field<F>(&mut self, f: F)
    where
        F: Fn(&mut LayoutField<T>),
    {
        match self {
            Resolve::LayoutField(l) => f(l),
            _ => {}
        }
    }

    pub fn is_set(&self) -> bool {
        match self {
            Resolve::UiValue(v) => v.is_set(),
            Resolve::LayoutField(l) => l.is_set(),
        }
    }

    pub fn is_auto(&self) -> bool {
        match self {
            Resolve::UiValue(v) => matches!(v, UiValue::Auto),
            Resolve::LayoutField(l) => l.is_auto(),
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            Resolve::UiValue(v) => matches!(v, UiValue::None),
            Resolve::LayoutField(l) => l.is_none(),
        }
    }

    pub fn is_unset(&self) -> bool {
        match self {
            Resolve::UiValue(v) => matches!(v, UiValue::Unset),
            Resolve::LayoutField(l) => l.is_unset(),
        }
    }

    pub fn merge_unset(&mut self, other: &Resolve<T>) {
        if self.is_unset() {
            *self = other.clone();
        }
    }

    pub fn merge_at_set(&mut self, to: &Resolve<T>) {
        if !to.is_unset() {
            *self = to.clone();
        }
    }

    pub fn resolve_just(&self) -> &T {
        match self {
            Resolve::UiValue(value) => match value {
                UiValue::Just(j) => Some(j),
                _ => None,
            },
            Resolve::LayoutField(field) => match &field.value {
                UiValue::Just(j) => Some(j),
                _ => None,
            },
        }
        .unwrap()
    }
}

impl<T: PartialOrd + Clone + 'static> From<UiValue<T>> for Resolve<T> {
    fn from(value: UiValue<T>) -> Self {
        Resolve::UiValue(value)
    }
}

impl<T: PartialOrd + Clone + 'static> From<LayoutField<T>> for Resolve<T> {
    fn from(value: LayoutField<T>) -> Self {
        Resolve::LayoutField(value)
    }
}

#[derive(Clone, Default)]
pub enum UiValue<T: Clone + 'static> {
    #[default]
    Unset,
    None,
    Auto,
    Inherit,
    Just(T),
    Measurement(Unit),
    Percent(f32),
}

impl<T: Savable + Clone + 'static> Savable for UiValue<T> {
    fn save(&self, saver: &mut impl Saver) {
        match self {
            UiValue::Unset => { 0u8.save(saver); }
            UiValue::None => { 1u8.save(saver); }
            UiValue::Auto => { 2u8.save(saver); }
            UiValue::Inherit => { 3u8.save(saver); }
            UiValue::Just(j) => {
                4u8.save(saver);
                j.save(saver);
            }
            UiValue::Measurement(u) => {
                5u8.save(saver);
                u.save(saver);
            }
            UiValue::Percent(p) => {
                6u8.save(saver);
                p.save(saver);
            }
        }
    }

    fn load(loader: &mut impl Loader) -> Result<Self, String> {
        let id = u8::load(loader)?;
        match id {
            0 => Ok(UiValue::Unset),
            1 => Ok(UiValue::None),
            2 => Ok(UiValue::Auto),
            3 => Ok(UiValue::Inherit),
            4 => Ok(UiValue::Just(T::load(loader)?)),
            5 => Ok(UiValue::Measurement(Unit::load(loader)?)),
            6 => Ok(UiValue::Percent(f32::load(loader)?)),
            _ => Err("Illegal id for UiValue when loading!".to_string())
        }
    }
}

impl<T: FromStr + Clone> FromStr for UiValue<T> {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let lower = value.trim().to_lowercase();

        match lower.as_str() {
            "unset" => Ok(UiValue::Unset),
            "none" => Ok(UiValue::None),
            "auto" => Ok(UiValue::Auto),
            "inherit" => Ok(UiValue::Inherit),
            _ => {
                if let Some(num_str) = lower.strip_suffix('%') {
                    let parsed = num_str
                        .trim()
                        .parse::<f32>()
                        .map_err(|_| format!("Invalid percentage: {}", value))?;
                    return Ok(UiValue::Percent(parsed / 100.0));
                }

                if let Ok(unit) = Unit::try_from(lower.clone()) {
                    return Ok(UiValue::Measurement(unit));
                }

                let t = T::from_str(value)
                    .map_err(|_| format!("{value} cannot be parsed into string!"))?;
                Ok(UiValue::Just(t))
            }
        }
    }
}

pub enum ResolveResult<T> {
    Value(T),
    Auto,
    None,
    UseDefault,
    Percent(f32),
}

impl<T: Clone> Clone for ResolveResult<T> {
    fn clone(&self) -> Self {
        match self {
            ResolveResult::Value(clone) => ResolveResult::Value(clone.clone()),
            ResolveResult::Auto => ResolveResult::Auto,
            ResolveResult::None => ResolveResult::None,
            ResolveResult::UseDefault => ResolveResult::UseDefault,
            ResolveResult::Percent(p) => ResolveResult::Percent(*p),
        }
    }
}

impl<T> ResolveResult<T> {
    pub fn unwrap(self) -> T {
        match self {
            Self::Value(t) => t,
            _ => panic!("Unwrapped empty UiValueResult!"),
        }
    }

    pub fn unwrap_or(self, or: T) -> T {
        match self {
            Self::Value(t) => t,
            _ => or,
        }
    }

    pub fn is_set(&self) -> bool {
        matches!(self, ResolveResult::Value(_))
    }

    pub fn is_none(&self) -> bool {
        matches!(self, ResolveResult::None)
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, ResolveResult::Auto)
    }

    pub fn is_use_default(&self) -> bool {
        matches!(self, ResolveResult::UseDefault)
    }

    pub fn is_percent(&self) -> bool {
        matches!(self, ResolveResult::Percent(_))
    }
}

impl<T: PartialOrd + Clone + 'static> ResolveResult<T> {
    pub fn unwrap_or_default(self, default: &Resolve<T>) -> T {
        match self {
            Self::Value(t) => t,
            _ => default.resolve_just().clone(),
        }
    }
}

impl ResolveResult<i32> {
    pub fn compute_percent(&self, parent: i32) -> i32 {
        match self {
            ResolveResult::Percent(p) => (*p * parent as f32) as i32,
            _ => parent,
        }
    }

    pub fn unwrap_or_default_or_percentage<F>(
        self,
        default: &Resolve<i32>,
        maybe_parent: Option<Rc<DangerousCell<UiElement>>>,
        map: F,
        sup: &impl InheritSupplier,
    ) -> i32
    where
        F: Fn(&dyn InheritSupplier) -> i32,
    {
        if self.is_percent() {
            return self.resolve_percent(maybe_parent, map, sup);
        }
        match self {
            Self::Value(t) => t,
            _ => default.resolve_just().clone(),
        }
    }

    pub fn resolve_percent<F>(
        &self,
        maybe_parent: Option<Rc<DangerousCell<UiElement>>>,
        map: F,
        sup: &impl InheritSupplier,
    ) -> i32
    where
        F: Fn(&dyn InheritSupplier) -> i32,
    {
        if let Some(parent) = maybe_parent {
            let binding = parent.get();
            let total = map(binding.state());
            self.compute_percent(total)
        } else {
            self.compute_percent(map(sup))
        }
    }
}

impl ResolveResult<f32> {
    pub fn compute_percent(&self, parent: f32) -> f32 {
        match self {
            ResolveResult::Percent(p) => *p * parent,
            _ => parent,
        }
    }

    pub fn unwrap_or_default_or_percentage<F>(
        self,
        default: &Resolve<f32>,
        maybe_parent: Option<Rc<DangerousCell<UiElement>>>,
        map: F,
        sup: &impl InheritSupplier,
    ) -> f32
    where
        F: Fn(&dyn InheritSupplier) -> f32,
    {
        if self.is_percent() {
            return self.resolve_percent(maybe_parent, map, sup);
        }
        match self {
            Self::Value(t) => t,
            _ => default.resolve_just().clone(),
        }
    }

    pub fn resolve_percent<F>(
        &self,
        maybe_parent: Option<Rc<DangerousCell<UiElement>>>,
        map: F,
        sup: &impl InheritSupplier,
    ) -> f32
    where
        F: Fn(&dyn InheritSupplier) -> f32,
    {
        if let Some(parent) = maybe_parent {
            let binding = parent.get();
            let total = map(binding.state());
            self.compute_percent(total)
        } else {
            self.compute_percent(map(sup))
        }
    }
}

impl<T: Clone + PartialOrd + 'static> UiValue<T> {
    pub fn to_field(self) -> LayoutField<T> {
        LayoutField::from(self)
    }

    pub fn to_resolve(self) -> Resolve<T> {
        Resolve::UiValue(self)
    }

    pub fn resolve<F>(
        &self,
        dpi: f32,
        parent: Option<Rc<DangerousCell<UiElement>>>,
        map: F,
    ) -> ResolveResult<T>
    where
        F: Fn(&UiStyle) -> &Self,
    {
        match self {
            UiValue::None => ResolveResult::None,
            UiValue::Auto => ResolveResult::Auto,
            UiValue::Inherit => {
                if parent.is_none() {
                    return ResolveResult::UseDefault;
                }
                let lock = parent.clone().unwrap();
                let guard = lock.get();
                map(guard.style()).resolve(dpi, parent.unwrap().get().state().parent.clone(), map)
            }
            UiValue::Just(v) => ResolveResult::Value(v.clone()),
            UiValue::Measurement(u) => {
                if TypeId::of::<T>() == TypeId::of::<i32>() {
                    unsafe {
                        let a = u.as_px(dpi);
                        ResolveResult::Value(Unsafe::cast_ref::<i32, T>(&a).clone())
                    }
                } else if TypeId::of::<T>() == TypeId::of::<f32>() {
                    unsafe {
                        let a = u.as_px(dpi) as f32;
                        ResolveResult::Value(Unsafe::cast_ref::<f32, T>(&a).clone())
                    }
                } else {
                    ResolveResult::UseDefault
                }
            }
            UiValue::Unset => {
                unsafe {
                    if parent.is_some() {
                        let cloned = parent.clone().unwrap();
                        let parent_value = Unsafe::cast_static(map(cloned.get().style()));
                        return parent_value.resolve(dpi, parent.clone(), map);
                    }
                }
                ResolveResult::UseDefault
            }
            UiValue::Percent(p) => ResolveResult::Percent(*p),
        }
    }
}

impl<T: Clone + 'static> UiValue<T> {
    pub fn is_set(&self) -> bool {
        !matches!(self, UiValue::None | UiValue::Auto | UiValue::Unset)
    }
}

//impl<T: Clone + 'static> PartialEq for UiValue<T> {
//    fn eq(&self, other: &Self) -> bool {
//        matches!(self, other)
//    }
//}

macro_rules! impl_interpolator_primitives {
    ($($t:ty,)*) => {
        $(
            impl Interpolator<$t> for $t {
                fn interpolate<E, F>(&mut self, start: &$t, end: &$t, percent: f32, _: &E, _: F)
                    where
                        E: UiElementStub,
                        F: Fn(&UiStyle) -> &Self,
                {
                    let frac = percent / 100.0;

                    *self = (*start as f32 + ((*end as f32 - *start as f32) * frac)) as $t;
                }
            }
        )*
    };
}

impl_interpolator_primitives!(
    u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64,
);

impl Default for UiStyle {
    fn default() -> Self {
        DEFAULT_STYLE.clone()
    }
}

impl UiStyle {
    pub fn stack_vertical() -> Self {
        let mut s = DEFAULT_STYLE.clone();
        modify_style!(s.direction = UiValue::Just(Direction::Vertical));
        modify_style!(s.child_align_x = UiValue::Just(ChildAlign::Middle));
        s
    }

    pub fn stack_horizontal() -> Self {
        let mut s = DEFAULT_STYLE.clone();
        modify_style!(s.direction = UiValue::Just(Direction::Horizontal));
        modify_style!(s.child_align_y = UiValue::Just(ChildAlign::Middle));
        s
    }
}

#[derive(Clone)]
pub struct ResCon {
    pub dpi: f32,
}

pub trait InheritSupplier {
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn paddings(&self) -> [i32; 4] {
        [0; 4]
    }
    fn margins(&self) -> [i32; 4] {
        [0; 4]
    }
    fn rotation(&self) -> f32 {
        0.0
    }
}

impl InheritSupplier for Window {
    fn x(&self) -> i32 {
        0
    }

    fn y(&self) -> i32 {
        0
    }

    fn width(&self) -> i32 {
        self.info.width as i32
    }

    fn height(&self) -> i32 {
        self.info.height as i32
    }
}
