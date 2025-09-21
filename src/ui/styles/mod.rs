pub mod enums;
pub mod groups;
pub mod interpolate;
pub mod types;
pub mod unit;

use crate::blanked_partial_ord;
use crate::color::RgbColor;
use crate::ui::elements::components::scroll;
use crate::ui::elements::{Element, UiElement, UiElementState, UiElementStub};
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
use std::fmt::{Debug, Formatter};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::str::FromStr;
use mvutils::save::{Loader, Savable, Saver};
use unit::Unit;
use crate::ui::geometry::SimpleRect;

lazy! {
    pub static DEFAULT_STYLE_INNER: UiStyleInner = UiStyleInner {
        x: UiValue::Just(0).to_field(),
        y: UiValue::Just(0).to_field(),
        width: UiValue::Auto.to_field(),
        height: UiValue::Auto.to_field(),
        padding: SideStyle::all(UiValue::Measurement(Unit::BeardFortnight(0.5))),
        margin: SideStyle::all(UiValue::Measurement(Unit::BeardFortnight(0.5))),
        origin: UiValue::Just(Origin::BottomLeft),
        position: UiValue::Just(Position::Relative),
        direction: UiValue::Just(Direction::Horizontal),
        child_align_x: UiValue::Just(ChildAlign::Start),
        child_align_y: UiValue::Just(ChildAlign::Start),
        background: ShapeStyle {
            resource: UiValue::Just(BasicInterpolatable::new(BackgroundRes::Color)),
            color: UiValue::Just(RgbColor::white()),
            texture: UiValue::None,
            shape: UiValue::Just(BasicInterpolatable::new(Geometry::Shape(MVR.shape.rect))),
            adaptive_ratio: UiValue::Just(1.0),
        },
        border: ShapeStyle {
            resource: UiValue::Just(BasicInterpolatable::new(BackgroundRes::Color)),
            color: UiValue::Just(RgbColor::black()),
            texture: UiValue::None,
            shape: UiValue::Just(BasicInterpolatable::new(Geometry::Adaptive(MVR.adaptive.void_rect))),
            //shape: UiValue::Just(BasicInterpolatable::new(UiShape::Shape(MVR.shape.rect))),
            adaptive_ratio: UiValue::Just(1.0),
        },
        detail: ShapeStyle {
            resource: UiValue::Just(BasicInterpolatable::new(BackgroundRes::Color)),
            color: UiValue::Just(RgbColor::black()),
            texture: UiValue::None,
            shape: UiValue::Just(BasicInterpolatable::new(Geometry::Shape(MVR.shape.rect))),
            adaptive_ratio: UiValue::Just(1.0),
        },
        text: TextStyle {
            size: UiValue::Measurement(Unit::BeardFortnight(1.0)),
            kerning: UiValue::Just(0.0),
            skew: UiValue::Just(0.0),
            stretch: UiValue::Just(Dimension::new(1.0, 1.0)),
            font: UiValue::Just(MVR.font.default),
            fit: UiValue::Just(TextFit::ExpandParent),
            color: UiValue::Just(RgbColor::black()),
            select_color: UiValue::Just(RgbColor::blue()),
            align_x: UiValue::Just(TextAlign::Middle),
            align_y: UiValue::Just(TextAlign::Middle),
        },
        transform: TransformStyle {
            translate: VectorField::splat(UiValue::Just(0)),
            scale: VectorField::splat(UiValue::Just(1.0)),
            rotate: UiValue::Just(0.0),
            origin: UiValue::Just(Origin::BottomLeft),
        },
        overflow_x: UiValue::Just(Overflow::Normal),
        overflow_y: UiValue::Just(Overflow::Normal),
        scrollbar: ScrollBarStyle {
            track: ShapeStyle {
                resource: UiValue::Just(BasicInterpolatable::new(BackgroundRes::Color)),
                color: UiValue::Just(scroll::OUTER_COLOR.clone()),
                texture: UiValue::None,
                shape: UiValue::Just(BasicInterpolatable::new(Geometry::Shape(MVR.shape.rect))),
                adaptive_ratio: UiValue::Just(1.0),
            },
            knob: ShapeStyle {
                resource: UiValue::Just(BasicInterpolatable::new(BackgroundRes::Color)),
                color: UiValue::Just(scroll::INNER_COLOR.clone()),
                texture: UiValue::None,
                shape: UiValue::Just(BasicInterpolatable::new(Geometry::Shape(MVR.shape.rect))),
                adaptive_ratio: UiValue::Just(1.0),
            },
            size: UiValue::Measurement(Unit::BeardFortnight(1.0)),
        }
    };

    pub static EMPTY_STYLE_INNER: UiStyleInner = UiStyleInner {
        x: UiValue::Unset.to_field(),
        y: UiValue::Unset.to_field(),
        width: UiValue::Unset.to_field(),
        height: UiValue::Unset.to_field(),
        padding: SideStyle::all(UiValue::Unset),
        margin: SideStyle::all(UiValue::Unset),
        origin: UiValue::Unset,
        position: UiValue::Unset,
        direction: UiValue::Unset,
        child_align_x: UiValue::Unset,
        child_align_y: UiValue::Unset,
        background: ShapeStyle {
            resource: UiValue::Unset,
            color: UiValue::Unset,
            texture: UiValue::Unset,
            shape: UiValue::Unset,
            adaptive_ratio: UiValue::Unset,
        },
        border: ShapeStyle {
            resource: UiValue::Unset,
            color: UiValue::Unset,
            texture: UiValue::Unset,
            shape: UiValue::Unset,
            adaptive_ratio: UiValue::Unset,
        },
        detail: ShapeStyle {
            resource: UiValue::Unset,
            color: UiValue::Unset,
            texture: UiValue::Unset,
            shape: UiValue::Unset,
            adaptive_ratio: UiValue::Unset,
        },
        text: TextStyle {
            size: UiValue::Unset,
            kerning: UiValue::Unset,
            skew: UiValue::Unset,
            stretch: UiValue::Unset,
            font: UiValue::Unset,
            fit: UiValue::Unset,
            color: UiValue::Unset,
            select_color: UiValue::Unset,
            align_x: UiValue::Unset,
            align_y: UiValue::Unset,
        },
        transform: TransformStyle {
            translate: VectorField::splat(UiValue::Unset),
            scale: VectorField::splat(UiValue::Unset),
            rotate: UiValue::Unset,
            origin: UiValue::Unset,
        },
        overflow_x: UiValue::Unset,
        overflow_y: UiValue::Unset,
        scrollbar: ScrollBarStyle {
            track: ShapeStyle {
                resource: UiValue::Unset,
                color: UiValue::Unset,
                texture: UiValue::Unset,
                shape: UiValue::Unset,
                adaptive_ratio: UiValue::Unset,
            },
            knob: ShapeStyle {
                resource: UiValue::Unset,
                color: UiValue::Unset,
                texture: UiValue::Unset,
                shape: UiValue::Unset,
                adaptive_ratio: UiValue::Unset,
            },
            //FUCK YOU WHY ARE YOU BREAKING WTF
            size: UiValue::Measurement(Unit::BeardFortnight(1.0)),
        }
    };

    pub static DEFAULT_STYLE: UiStyle = UiStyle {
        base: DEFAULT_STYLE_INNER.clone(),
        transition_duration: UiValue::Just(0),
        hover: EMPTY_STYLE_INNER.clone()
    };

    pub static EMPTY_STYLE: UiStyle = UiStyle {
        base: EMPTY_STYLE_INNER.clone(),
        transition_duration: UiValue::Unset,
        hover: EMPTY_STYLE_INNER.clone()
    };
}

/// Resolves the given style field by using the `UiElement` itself
#[macro_export]
macro_rules! resolve {
    ($elem:ident, $($style:ident).*) => {
        {
            let s = &$elem.style().$($style).*;
            let state = $elem.state();
            let body = $elem.body();
            crate::ui::utils::resolve_value(s, state, body, |s| &s.$($style).*)
        }
    };
}

/// Resolves the given style field by using the `UiElementState`, `ElementBody` and `UiStyle`
#[macro_export]
macro_rules! resolve2 {
    ($elem_state:ident, $elem_body:ident, $style_ident:ident.$($style:ident).*) => {
        {
            crate::ui::utils::resolve_value(&$style_ident.$($style).*, $elem_state, $elem_body, |s| &s.$($style).*)
        }
    };
}

/// Resolves the given LayoutField field by using the elem
#[macro_export]
macro_rules! resolve3 {
    ($elem:ident, $($style:ident).*, $self_value:expr, $sup:ident, $sup_a:expr) => {
        {
            let s = &$elem.style().$($style).*;
            let state = $elem.state();
            let body = $elem.body();
            crate::ui::utils::resolve_field(s, state, body, |s| &s.$($style).*, $sup, $sup_a, $self_value)
        }
    };
}

//this macro solely exists so the other macros dont break LOL
#[macro_export]
macro_rules! modify_style {
    ($($style:ident).* = $($ac:tt)*) => {
        $($style).* = $($ac)*;
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

pub struct UiStyleWriteObserver<'a> {
    inner: &'a mut UiStyle,
    invalid_flag: &'a mut u8,
    di: bool
}

impl<'a> UiStyleWriteObserver<'a> {
    pub fn new(s: &'a mut UiStyle, invalid_flag: &'a mut u8) -> Self {
        Self {
            inner: s,
            invalid_flag,
            di: false,
        }
    }

    pub fn disable_invalidation(&mut self) {
        self.di = true;
    }
}

impl Deref for UiStyleWriteObserver<'_> {
    type Target = UiStyle;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl DerefMut for UiStyleWriteObserver<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

impl Drop for UiStyleWriteObserver<'_> {
    fn drop(&mut self) {
        if !self.di {
            *self.invalid_flag = UiElementState::FRAMES_TO_BE_INVALID;
        }
    }
}

#[derive(Clone, Debug)]
pub struct UiStyleInner {
    pub x: LayoutField<i32>,
    pub y: LayoutField<i32>,
    pub width: LayoutField<i32>,
    pub height: LayoutField<i32>,
    pub padding: SideStyle,
    pub margin: SideStyle,
    pub origin: UiValue<Origin>,
    pub position: UiValue<Position>,
    pub direction: UiValue<Direction>,
    pub child_align_x: UiValue<ChildAlign>,
    pub child_align_y: UiValue<ChildAlign>,

    pub background: ShapeStyle,
    pub border: ShapeStyle,
    pub detail: ShapeStyle,

    pub text: TextStyle,
    pub transform: TransformStyle,

    pub overflow_x: UiValue<Overflow>,
    pub overflow_y: UiValue<Overflow>,
    pub scrollbar: ScrollBarStyle,
}

#[derive(Clone, Debug)]
pub struct UiStyle {
    base: UiStyleInner,
    //unused atm
    pub transition_duration: UiValue<i32>, //just is in ms
    pub hover: UiStyleInner,
}

impl Deref for UiStyle {
    type Target = UiStyleInner;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for UiStyle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl From<UiStyleInner> for UiStyle {
    fn from(value: UiStyleInner) -> Self {
        Self {
            base: value,
            transition_duration: DEFAULT_STYLE.transition_duration.clone(),
            hover: EMPTY_STYLE_INNER.clone(),
        }
    }
}

unsafe impl Sync for UiStyle {}
unsafe impl Send for UiStyle {}

impl UiStyleInner {
    pub fn merge_unset(&mut self, other: &UiStyleInner) {
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
        self.detail.merge_unset(&other.detail);
        self.text.merge_unset(&other.text);
        self.transform.merge_unset(&other.transform);
        self.overflow_x.merge_unset(&other.overflow_x);
        self.overflow_y.merge_unset(&other.overflow_y);
        self.scrollbar.merge_unset(&other.scrollbar);
    }

    pub fn merge_at_set_of(&mut self, to: &UiStyleInner) {
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
        self.detail.merge_at_set(&to.detail);
        self.text.merge_at_set(&to.text);
        self.transform.merge_at_set(&to.transform);
        self.overflow_x.merge_at_set(&to.overflow_x);
        self.overflow_y.merge_at_set(&to.overflow_y);
        self.scrollbar.merge_at_set(&to.scrollbar);
    }
}

impl UiStyle {
    pub fn merge_unset(&mut self, other: &UiStyle) {
        self.base.merge_unset(&other.base);
        self.transition_duration.merge_unset(&other.transition_duration);
    }

    pub fn merge_at_set_of(&mut self, to: &UiStyle) {
        self.base.merge_at_set_of(&to.base);
        self.transition_duration.merge_at_set(&to.transition_duration);
    }

    pub fn get_hover(&self) -> UiStyle {
        let mut s = self.base.clone();
        s.merge_at_set_of(&self.hover);
        let mut s: UiStyle = s.into();
        s.transition_duration = self.transition_duration.clone();
        s
    }
}

blanked_partial_ord!(UiStyle);
blanked_partial_ord!(UiStyleInner);

blanked_partial_ord!(TextStyle);

blanked_partial_ord!(SideStyle);

#[derive(Clone, Default, Debug)]
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

impl<T> Debug for ResolveResult<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveResult::Value(_) => f.write_str("value"),
            ResolveResult::Auto => f.write_str("auto"),
            ResolveResult::None => f.write_str("none"),
            ResolveResult::UseDefault => f.write_str("default"),
            ResolveResult::Percent(_) => f.write_str("percent"),
        }
    }
}

impl<T: PartialOrd + Clone + Sized + 'static> ResolveResult<T> {
    pub fn unwrap_or_default(self, default: &UiValue<T>) -> T {
        match self {
            Self::Value(t) => t,
            _ => default.resolve_just().clone(),
        }
    }

    pub fn compute_percent(&self, parent: T) -> T {
        match self {
            ResolveResult::Percent(p) => {
                unsafe {
                    //this will be fine dw
                    if TypeId::of::<T>() == TypeId::of::<i32>() {
                        let parent_i32 = *( &parent as *const T as *const i32 );
                        let r = (*p * parent_i32 as f32) as i32;
                        mem::transmute_copy::<i32, T>(&r)
                    } else if TypeId::of::<T>() == TypeId::of::<f32>() {
                        let parent_f32 = *( &parent as *const T as *const f32 );
                        let r = (*p * parent_f32);
                        mem::transmute_copy::<f32, T>(&r)
                    } else {
                        parent
                    }
                }
            },
            _ => parent,
        }
    }

    pub fn unwrap_or_default_or_percentage<F>(
        self,
        default: &UiValue<T>,
        maybe_parent: Option<Element>,
        map: F,
        sup: &dyn InheritSupplier,
    ) -> T
    where
        F: Fn(&dyn InheritSupplier) -> T,
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
        maybe_parent: Option<Element>,
        map: F,
        sup: &dyn InheritSupplier,
    ) -> T
    where
        F: Fn(&dyn InheritSupplier) -> T,
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

    pub fn resolve<F>(
        &self,
        dpi: f32,
        parent: Option<Element>,
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
                        let a = u.resolve(dpi);
                        ResolveResult::Value(Unsafe::cast_ref::<i32, T>(&a).clone())
                    }
                } else if TypeId::of::<T>() == TypeId::of::<f32>() {
                    unsafe {
                        let a = u.resolve(dpi) as f32;
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
                        let parent_value = Unsafe::cast_lifetime(map(cloned.get().style()));
                        return parent_value.resolve(dpi, parent.clone(), map);
                    }
                }
                ResolveResult::UseDefault
            }
            UiValue::Percent(p) => ResolveResult::Percent(*p),
        }
    }

    pub fn resolve_just(&self) -> &T {
        enum_val_ref!(UiValue, self, Just)
    }
}

impl<T: Clone + 'static> UiValue<T> {
    pub fn is_set(&self) -> bool {
        !matches!(self, UiValue::None | UiValue::Auto | UiValue::Unset)
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, UiValue::Auto)
    }

    pub fn is_none(&self) -> bool {
        matches!(self, UiValue::None)
    }

    pub fn is_unset(&self) -> bool {
        matches!(self, UiValue::Unset)
    }

    pub fn merge_unset(&mut self, other: &UiValue<T>) {
        if self.is_unset() {
            *self = other.clone();
        }
    }

    pub fn merge_at_set(&mut self, to: &UiValue<T>) {
        if !to.is_unset() {
            *self = to.clone();
        }
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

    pub fn inheriting() -> Self {
        let shape = ShapeStyle {
            resource: UiValue::Inherit,
            color: UiValue::Inherit,
            texture: UiValue::Inherit,
            shape: UiValue::Inherit,
            adaptive_ratio: UiValue::Inherit,
        };

        let trans = TransformStyle {
            translate: VectorField {
                x: UiValue::Inherit,
                y: UiValue::Inherit
            },
            scale: VectorField {
                x: UiValue::Inherit,
                y: UiValue::Inherit
            },
            rotate: UiValue::Inherit,
            origin: UiValue::Inherit,
        };

        let inner_inherit = UiStyleInner {
            x: UiValue::Inherit.to_field(),
            y: UiValue::Inherit.to_field(),
            width: UiValue::Inherit.to_field(),
            height: UiValue::Inherit.to_field(),
            padding: SideStyle {
                top: UiValue::Inherit.to_field(),
                bottom: UiValue::Inherit.to_field(),
                left: UiValue::Inherit.to_field(),
                right: UiValue::Inherit.to_field(),
            },
            margin: SideStyle {
                top: UiValue::Inherit.to_field(),
                bottom: UiValue::Inherit.to_field(),
                left: UiValue::Inherit.to_field(),
                right: UiValue::Inherit.to_field(),
            },
            origin: UiValue::Inherit,
            position: UiValue::Inherit,
            direction: UiValue::Inherit,
            child_align_x: UiValue::Inherit,
            child_align_y: UiValue::Inherit,
            background: shape.clone(),
            border: shape.clone(),
            detail: shape.clone(),
            text: TextStyle {
                size: UiValue::Inherit,
                kerning: UiValue::Inherit,
                skew: UiValue::Inherit,
                stretch: UiValue::Inherit,
                font: UiValue::Inherit,
                fit: UiValue::Inherit,
                color: UiValue::Inherit,
                select_color: UiValue::Inherit,
                align_x: UiValue::Inherit,
                align_y: UiValue::Inherit,
            },
            transform: trans.clone(),
            overflow_x: UiValue::Inherit,
            overflow_y: UiValue::Inherit,
            scrollbar: ScrollBarStyle {
                track: shape.clone(),
                knob: shape,
                size: UiValue::Inherit,
            },
        };

        Self {
            base: inner_inherit.clone(),
            transition_duration: UiValue::Inherit,
            hover: inner_inherit,
        }
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

    fn area(&self) -> SimpleRect {
        let x = self.x();
        let y = self.y();
        let w = self.width();
        let h = self.height();
        SimpleRect::new(x, y, w, h)
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
