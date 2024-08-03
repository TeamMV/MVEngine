use crate::render::color::{Color, Fmt, RgbColor};
use crate::render::text::Font;
use crate::resources::resources::R;
use crate::ui::background::{Background, BackgroundInfo, RectangleBackground};
use crate::ui::elements::UiElement;
use crate::{blanked_partial_ord, fast_partial_ord};
use mvutils::save::Savable;
use mvutils::unsafe_utils::Unsafe;
use mvutils::utils::{PClamp, TetrahedronOp};
use mvutils::{enum_val, enum_val_ref};
use num_traits::Num;
use parking_lot::RwLock;
use std::any::Any;
use std::cmp::Ordering;
use std::convert::Infallible;
use std::fmt::Debug;
use std::mem;
use std::ops::{Add, Deref, Mul, Sub};
use std::sync::Arc;

#[macro_export]
macro_rules! resolve {
    ($elem:ident, $($style:ident).*) => {
        {
            let s = &$elem.style().$($style).*;
            let v: Option<_> = s.resolve($elem.state().ctx.dpi, $elem.state().parent.clone(), |s| {&s.$($style).*});
            if let Some(v) = v {
                v
            }
            else {
                log::error!("UiValue {:?} failed to resolve on element {:?}", stringify!($($style).*), $elem.attributes().id);
                $crate::ui::styles::UiStyle::default().$($style).*
                .resolve($elem.state().ctx.dpi, None, |s| {&s.$($style).*})
                .expect("Default style could not be resolved")
            }
        }
    };
}

#[macro_export]
macro_rules! interpolator_map_fn {
    ($s:ident, $func:ident, $elem:ident, $($style:ident).*) => {
        |s| {
                let f_res = $func(s);
                let tmp = $elem.read();
                let resolved = f_res
                    .resolve(tmp.state().ctx.dpi, tmp.state().parent.clone(), |uis| &$func(uis))
                    .unwrap_or_else(|_| panic!("Could not resolve UiStyle field {}!", $($s.$style).*));
                return resolved.$($style).*;
            }
    };
}

#[macro_export]
macro_rules! modify_style {
    ($($style:ident).* = $($ac:tt)*) => {
        $($style).*.for_value(|v| *v = $($ac)*);
    };
    ($($style:ident).*:$acc:ident = $($ac:tt)*) => {
        $($style).*.for_field(|l| (*l).$acc = $($ac)*);
    };
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
    pub rotation_origin: Resolve<Origin>,
    pub rotation: Resolve<f32>,
    pub direction: Resolve<Direction>,
    pub child_align: Resolve<ChildAlign>,

    pub text: TextStyle,

    pub background: BackgroundInfo,
}

blanked_partial_ord!(UiStyle);

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
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

    pub fn get_actual_x(&self, x: i32, width: i32) -> i32 {
        match self {
            Origin::TopLeft => x,
            Origin::BottomLeft => x,
            Origin::TopRight => x - width,
            Origin::BottomRight => x - width,
            Origin::Center => x - width / 2,
            Origin::Custom(cx, _) => x - cx,
        }
    }

    pub fn get_actual_y(&self, y: i32, height: i32) -> i32 {
        match self {
            Origin::TopLeft => y - height,
            Origin::BottomLeft => y,
            Origin::TopRight => y - height,
            Origin::BottomRight => y,
            Origin::Center => y - height / 2,
            Origin::Custom(_, cy) => y - cy,
        }
    }
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum Position {
    Absolute,
    #[default]
    Relative,
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum Direction {
    Vertical,
    #[default]
    Horizontal,
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum TextFit {
    ExpandParent,
    #[default]
    CropText,
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum ChildAlign {
    #[default]
    Start,
    End,
    Middle,
    OffsetStart(i32),
    OffsetEnd(i32),
    OffsetMiddle(i32),
}

#[derive(Clone)]
pub struct TextStyle {
    pub size: Resolve<f32>,
    pub kerning: Resolve<f32>,
    pub skew: Resolve<f32>,
    pub stretch: Resolve<Dimension<f32>>,
    pub font: Resolve<Arc<Font>>,
    pub fit: Resolve<TextFit>,
    pub color: Resolve<RgbColor>
}

blanked_partial_ord!(TextStyle);

impl TextStyle {
    pub fn initial() -> Self {
        Self {
            size: UiValue::Measurement(Unit::BarleyCorn(1.0))
                .to_field()
                .into(),
            kerning: UiValue::None.to_field().into(),
            skew: UiValue::None.to_field().into(),
            stretch: UiValue::None.to_field().into(),
            font: UiValue::Auto.into(),
            fit: UiValue::Auto.into(),
            color: UiValue::Auto.into(),
        }
    }
}

#[derive(Clone)]
pub struct LayoutField<T: PartialOrd + Clone + 'static> {
    pub value: UiValue<T>,
    pub min: UiValue<T>,
    pub max: UiValue<T>,
}

impl<T: PartialOrd + Clone> LayoutField<T> {
    pub fn to_resolve(self) -> Resolve<T> {
        Resolve::LayoutField(self)
    }

    fn resolve<F>(&self, dpi: f32, parent: Option<Arc<RwLock<dyn UiElement>>>, map: F) -> Option<T>
    where
        F: Fn(&UiStyle) -> &Self,
    {
        let value = self.value.resolve(dpi, parent.clone(), |s| &map(s).value);
        let min = self.min.resolve(dpi, parent.clone(), |s| &map(s).min);
        let max = self.max.resolve(dpi, parent, |s| &map(s).max);

        if value.is_none() {
            return None;
        }

        let mut emin = None;
        let mut emax = None;

        if min.is_none() {
            emin = Some(value.clone().unwrap());
        } else {
            emin = min;
        }

        if max.is_none() {
            emax = Some(value.clone().unwrap());
        } else {
            emax = max;
        }

        Some(value.unwrap().p_clamp(emin.unwrap(), emax.unwrap()))
    }
}

impl<T: PartialOrd + Clone> From<UiValue<T>> for LayoutField<T> {
    fn from(value: UiValue<T>) -> Self {
        LayoutField {
            value,
            min: UiValue::None,
            max: UiValue::None,
        }
    }
}

impl<T: PartialOrd + Clone> LayoutField<T> {
    pub fn is_set(&self) -> bool {
        return self.value.is_set();
    }

    pub fn is_none(&self) -> bool {
        return matches!(self.value, UiValue::None);
    }

    pub fn is_auto(&self) -> bool {
        return matches!(self.value, UiValue::Auto);
    }

    pub fn is_min_set(&self) -> bool {
        return self.min.is_set();
    }

    pub fn is_min_none(&self) -> bool {
        return matches!(self.min, UiValue::None);
    }

    pub fn is_min_auto(&self) -> bool {
        return matches!(self.min, UiValue::Auto);
    }

    pub fn is_max_set(&self) -> bool {
        return self.max.is_set();
    }

    pub fn is_max_none(&self) -> bool {
        return matches!(self.max, UiValue::None);
    }

    pub fn is_max_auto(&self) -> bool {
        return matches!(self.max, UiValue::Auto);
    }

    pub fn apply<F>(&self, value: T, elem: &dyn UiElement, map: F) -> T
    where
        F: Fn(&UiStyle) -> &Self,
    {
        let min = self
            .min
            .resolve(elem.state().ctx.dpi, elem.state().parent.clone(), |s| {
                &map(s).min
            });
        let max = self
            .max
            .resolve(elem.state().ctx.dpi, elem.state().parent.clone(), |s| {
                &map(s).max
            });

        let mut ret = value;

        if min.is_some() {
            let min = min.unwrap();
            if ret < min {
                ret = min;
            }
        }

        if max.is_some() {
            let max = max.unwrap();
            if ret > max {
                ret = max;
            }
        }

        ret
    }
}

#[derive(Clone, Debug)]
pub struct Dimension<T: Num + Clone + Debug> {
    pub width: T,
    pub height: T,
}

impl<T: PartialOrd + Num + Clone + Debug> PartialEq<Self> for Dimension<T> {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width && self.height == other.height
    }
}

impl<T: PartialOrd + Num + Clone + Debug> PartialOrd for Dimension<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.width
            .partial_cmp(&other.width)
            .or_else(|| self.height.partial_cmp(&other.height))
    }
}

impl<T: Num + Clone + Debug> Dimension<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    pub fn components(&self) -> (T, T) {
        (self.width.clone(), self.height.clone())
    }
}

#[derive(Clone, Debug)]
pub struct Point<T: Num + Clone + Debug> {
    pub x: T,
    pub y: T,
}

impl<T: PartialOrd + Num + Clone + Debug> PartialEq<Self> for Point<T> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl<T: PartialOrd + Num + Clone + Debug> PartialOrd for Point<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.x
            .partial_cmp(&other.x)
            .or_else(|| self.y.partial_cmp(&other.y))
    }
}

impl<T: Num + Clone + Debug> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn components(&self) -> (T, T) {
        (self.x.clone(), self.y.clone())
    }
}

#[derive(Clone, Debug)]
pub struct Location<T: Num + Clone + Debug> {
    pub x: T,
    pub y: T,
    pub dimension: Dimension<T>,
    pub rotation: f32,
    pub origin: Point<T>,
}

impl<T: Num + Clone + Debug> Location<T> {
    pub fn new(x: T, y: T, dimension: Dimension<T>, rotation: f32, origin: Point<T>) -> Self {
        Self {
            x,
            y,
            dimension,
            rotation,
            origin,
        }
    }

    pub fn simple(x: T, y: T, dimension: Dimension<T>) -> Self {
        let (w, h) = dimension.components();
        Self {
            x,
            y,
            dimension,
            rotation: 0.0,
            origin: Point::new(w / T::one().add(T::one()), h / T::one().add(T::one())),
        }
    }
}

#[derive(Clone)]
pub struct SideStyle {
    pub top: Resolve<i32>,
    pub bottom: Resolve<i32>,
    pub left: Resolve<i32>,
    pub right: Resolve<i32>,
}

blanked_partial_ord!(SideStyle);

impl SideStyle {
    pub fn all_i32(v: i32) -> Self {
        Self {
            top: UiValue::Just(v).into(),
            bottom: UiValue::Just(v).into(),
            left: UiValue::Just(v).into(),
            right: UiValue::Just(v).into(),
        }
    }

    pub fn all(v: UiValue<i32>) -> Self {
        Self {
            top: v.clone().into(),
            bottom: v.clone().into(),
            left: v.clone().into(),
            right: v.into(),
        }
    }

    pub fn set(&mut self, v: UiValue<i32>) {
        self.top = v.clone().into();
        self.bottom = v.clone().into();
        self.left = v.clone().into();
        self.right = v.into();
    }

    pub fn get<F>(&self, elem: &dyn UiElement, map: F) -> [i32; 4]
    where
        F: Fn(&UiStyle) -> &Self,
    {
        let top = if self.top.is_set() {
            self.top
                .resolve(elem.state().ctx.dpi, elem.state().parent.clone(), |s| {
                    &map(s).top
                })
                .unwrap()
        } else {
            self.top.is_auto().yn(5, 0)
        };
        let bottom = if self.bottom.is_set() {
            self.bottom
                .resolve(elem.state().ctx.dpi, elem.state().parent.clone(), |s| {
                    &map(s).bottom
                })
                .unwrap()
        } else {
            self.bottom.is_auto().yn(5, 0)
        };
        let left = if self.left.is_set() {
            self.left
                .resolve(elem.state().ctx.dpi, elem.state().parent.clone(), |s| {
                    &map(s).left
                })
                .unwrap()
        } else {
            self.left.is_auto().yn(5, 0)
        };
        let right = if self.right.is_set() {
            self.right
                .resolve(elem.state().ctx.dpi, elem.state().parent.clone(), |s| {
                    &map(s).right
                })
                .unwrap()
        } else {
            self.right.is_auto().yn(5, 0)
        };
        [top, bottom, left, right]
    }
}

#[derive(Clone)]
pub enum Resolve<T: PartialOrd + Clone + 'static> {
    UiValue(UiValue<T>),
    LayoutField(LayoutField<T>),
}

impl<T: PartialOrd + Clone + 'static> Resolve<T> {
    pub fn resolve<F>(
        &self,
        dpi: f32,
        parent: Option<Arc<RwLock<dyn UiElement>>>,
        map: F,
    ) -> Option<T>
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
            Resolve::UiValue(ref mut v) => f(v),
            Resolve::LayoutField(ref mut l) => f(&mut l.value),
        }
    }

    pub fn for_field<F>(&mut self, f: F)
    where
        F: Fn(&mut LayoutField<T>),
    {
        match self {
            Resolve::LayoutField(ref mut l) => f(l),
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
    None,
    Auto,
    Inherit,
    Clone(Arc<dyn UiElement>),
    Just(T),
    Measurement(Unit),
}

impl<T: Clone + PartialOrd + 'static> UiValue<T> {
    pub fn to_field(self) -> LayoutField<T> {
        LayoutField::from(self)
    }

    fn resolve<F>(&self, dpi: f32, parent: Option<Arc<RwLock<dyn UiElement>>>, map: F) -> Option<T>
    where
        F: Fn(&UiStyle) -> &Self,
    {
        match self {
            UiValue::None => None,
            UiValue::Auto => None,
            UiValue::Inherit => {
                let lock = parent.clone().unwrap_or_else(no_parent);
                let guard = lock.read();
                map(guard.style()).resolve(
                    dpi,
                    Some(
                        parent
                            .unwrap_or_else(no_parent)
                            .read()
                            .state()
                            .parent
                            .clone()
                            .unwrap_or_else(no_parent)
                            .clone(),
                    ),
                    map,
                )
            }
            UiValue::Clone(e) => map(e.style()).resolve(dpi, e.state().parent.clone(), map),
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
}

impl<T: Clone + 'static> UiValue<T> {
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

pub trait Interpolator<T: PartialOrd + Clone + 'static> {
    fn interpolate<E, F>(&mut self, end: &Self, percent: f32, elem: Arc<RwLock<E>>, f: F)
    where
        E: UiElement + ?Sized,
        F: Fn(&UiStyle) -> &Self;
}

macro_rules! impl_interpolator_primitives {
    ($($t:ty,)*) => {
        $(
            impl Interpolator<$t> for $t {
                fn interpolate<E, F>(&mut self, end: &$t, percent: f32, elem: Arc<RwLock<E>>, f: F)
                    where
                        E: UiElement + ?Sized,
                        F: Fn(&UiStyle) -> &Self,
                {
                    let frac = percent / 100.0;
                    *self = *self + ((((*end - *self) as f32) * frac) as $t);
                }
            }
        )*
    };
}

impl_interpolator_primitives!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64,);

impl<T: PartialOrd + Clone + 'static + Interpolator<T>> Interpolator<T> for Resolve<T> {
    fn interpolate<E, F>(&mut self, end: &Self, percent: f32, elem: Arc<RwLock<E>>, f: F)
    where
        E: UiElement + ?Sized,
        F: Fn(&UiStyle) -> &Self,
    {
        let guard = elem.read();
        let state = guard.state();

        let mut start_resolve = self.resolve(state.ctx.dpi, state.parent.clone(), |s| f(s));
        let end_resolve = end.resolve(state.ctx.dpi, state.parent.clone(), |s| f(s));

        if start_resolve.is_none() || end_resolve.is_none() {
            return;
        }

        let mut start_resolve = start_resolve.unwrap();
        let end_resolve = end_resolve.unwrap();

        let dpi = state.ctx.dpi;
        let parent = state.parent.clone();

        drop(guard);

        //why do i have to clone parent again???
        unsafe {
            start_resolve.interpolate(&end_resolve, percent, elem, move |s| {
                Unsafe::cast_static(&f(s).resolve(dpi, parent.clone(), |s2| &f(s2)).unwrap())
            });

            *self = Resolve::UiValue(UiValue::Just(start_resolve));
        }
    }
}

impl Interpolator<UiStyle> for UiStyle {
    fn interpolate<E, F>(&mut self, end: &UiStyle, percent: f32, elem: Arc<RwLock<E>>, f: F)
    where
        E: UiElement + ?Sized,
        F: Fn(&UiStyle) -> &Self,
    {
        self.x.interpolate(&end.x, percent, elem.clone(), |s| &s.x);
        self.y.interpolate(&end.y, percent, elem.clone(), |s| &s.y);
        self.width.interpolate(&end.width, percent, elem.clone(), |s| &s.width);
        self.height.interpolate(&end.height, percent, elem.clone(), |s| &s.height);
        self.padding.left.interpolate(&end.padding.left, percent, elem.clone(), |s| { &s.padding.left });
        self.padding.right.interpolate(&end.padding.right, percent, elem.clone(), |s| { &s.padding.right });
        self.padding.top.interpolate(&end.padding.top, percent, elem.clone(), |s| &s.padding.top);
        self.padding.bottom.interpolate(&end.padding.bottom, percent, elem.clone(), |s| { &s.padding.bottom });

        self.margin.left.interpolate(&end.margin.left, percent, elem.clone(), |s| &s.margin.left);
        self.margin.right.interpolate(&end.margin.right, percent, elem.clone(), |s| { &s.margin.right });
        self.margin.top.interpolate(&end.margin.top, percent, elem.clone(), |s| &s.margin.top);
        self.margin.bottom.interpolate(&end.margin.bottom, percent, elem.clone(), |s| { &s.margin.bottom });

        self.origin = (percent < 50f32).yn(self.origin.clone(), end.origin.clone());
        self.position = (percent < 50f32).yn(self.position.clone(), end.position.clone());
        self.rotation_origin =
            (percent < 50f32).yn(self.rotation_origin.clone(), end.rotation_origin.clone());
        self.rotation
            .interpolate(&end.rotation, percent, elem.clone(), |s| &s.rotation);
        self.direction = (percent < 50f32).yn(self.direction.clone(), end.direction.clone());
        self.child_align = (percent < 50f32).yn(self.child_align.clone(), end.child_align.clone());

        self.text.size.interpolate(&end.text.size, percent, elem.clone(), |s| { &s.text.size });
        self.text.stretch.interpolate(&end.text.stretch, percent, elem.clone(), |s| { &s.text.stretch });
        self.text.skew.interpolate(&end.text.skew, percent, elem.clone(), |s| { &s.text.skew });
        self.text.kerning.interpolate(&end.text.kerning, percent, elem.clone(), |s| { &s.text.kerning });
        self.text.color.interpolate(&end.text.color, percent, elem.clone(), |s| { &s.text.color });
        self.text.fit = (percent < 50f32).yn(self.text.fit.clone(), end.text.fit.clone());
        self.text.font = (percent < 50f32).yn(self.text.font.clone(), end.text.font.clone());

        self.background.border_width.interpolate(
            &end.background.border_width,
            percent,
            elem.clone(),
            |s| &s.background.border_width,
        );
        self.background.main_color.interpolate(
            &end.background.main_color,
            percent,
            elem.clone(),
            |s| &s.background.main_color,
        );
        self.background.border_color.interpolate(
            &end.background.border_color,
            percent,
            elem.clone(),
            |s| &s.background.border_color,
        );
    }
}

//impl Interpolator<SideStyle> for SideStyle {
//    fn interpolate<E, F>(&mut self, end: &SideStyle, percent: f32, elem: Arc<RwLock<E>>, f: F)
//    where
//        E: UiElement,
//        F: Fn(&UiStyle) -> Resolve<SideStyle>,
//    {
//        self.top.interpolate(
//            &end.top,
//            percent,
//            elem.clone(),
//            interpolator_map_fn!(self, f, elem, top),
//        );
//        self.bottom.interpolate(
//            &end.bottom,
//            percent,
//            elem.clone(),
//            interpolator_map_fn!(self, f, elem, bottom),
//        );
//        self.left.interpolate(
//            &end.left,
//            percent,
//            elem.clone(),
//            interpolator_map_fn!(self, f, elem, left),
//        );
//        self.right.interpolate(
//            &end.right,
//            percent,
//            elem.clone(),
//            interpolator_map_fn!(self, f, elem, right),
//        );
//    }
//}

impl Interpolator<TextStyle> for TextStyle {
    fn interpolate<E, F>(&mut self, end: &TextStyle, percent: f32, elem: Arc<RwLock<E>>, f: F)
    where
        E: UiElement + ?Sized,
        F: Fn(&UiStyle) -> &Self,
    {
        self.fit = (percent < 50f32).yn(self.fit.clone(), end.fit.clone());
        self.size
            .interpolate(&end.size, percent, elem.clone(), |s| &s.text.size);
        self.font = (percent < 50f32).yn(self.font.clone(), end.font.clone());
        self.kerning
            .interpolate(&end.kerning, percent, elem.clone(), |s| &s.text.kerning);
        self.skew
            .interpolate(&end.skew, percent, elem.clone(), |s| &s.text.skew);
        self.stretch
            .interpolate(&end.stretch, percent, elem.clone(), |s| &s.text.stretch);
    }
}

impl<T: Interpolator<T> + Num + Clone + Debug + PartialOrd + 'static> Interpolator<Dimension<T>>
    for Dimension<T>
{
    fn interpolate<E, F>(&mut self, end: &Dimension<T>, percent: f32, elem: Arc<RwLock<E>>, f: F)
    where
        E: UiElement + ?Sized,
        F: Fn(&UiStyle) -> &Self,
    {
        self.width
            .interpolate(&end.width, percent, elem.clone(), |s| &f(s).width);
        self.height
            .interpolate(&end.height, percent, elem.clone(), |s| &f(s).height);
    }
}

impl<T: Interpolator<T> + Num + Clone + Debug + PartialOrd + 'static> Interpolator<Point<T>>
    for Point<T>
{
    fn interpolate<E, F>(&mut self, end: &Point<T>, percent: f32, elem: Arc<RwLock<E>>, f: F)
    where
        E: UiElement + ?Sized,
        F: Fn(&UiStyle) -> &Self,
    {
        self.x
            .interpolate(&end.x, percent, elem.clone(), |s| &f(s).x);
        self.y
            .interpolate(&end.y, percent, elem.clone(), |s| &f(s).y);
    }
}

impl<Fm: Fmt + 'static, T: Interpolator<T> + PartialOrd + Default + Clone + 'static>
    Interpolator<Color<Fm, T>> for Color<Fm, T>
{
    fn interpolate<E, F>(&mut self, end: &Color<Fm, T>, percent: f32, elem: Arc<RwLock<E>>, f: F)
    where
        E: UiElement + ?Sized,
        F: Fn(&UiStyle) -> &Self,
    {
        self.c1
            .interpolate(&end.c1, percent, elem.clone(), |s| &f(s).c1);
        self.c2
            .interpolate(&end.c2, percent, elem.clone(), |s| &f(s).c2);
        self.c3
            .interpolate(&end.c3, percent, elem.clone(), |s| &f(s).c3);
        self.c4
            .interpolate(&end.c4, percent, elem.clone(), |s| &f(s).c4);
    }
}

impl Default for UiStyle {
    fn default() -> Self {
        Self {
            x: UiValue::Just(0).to_field().into(),
            y: UiValue::Just(0).to_field().into(),
            width: UiValue::Auto.to_field().into(),
            height: UiValue::Auto.to_field().into(),
            padding: SideStyle::all_i32(0).into(),
            margin: SideStyle::all_i32(0).into(),
            origin: UiValue::Just(Origin::BottomLeft).into(),
            position: UiValue::Just(Position::Relative).into(),
            rotation_origin: UiValue::Just(Origin::Center).into(),
            rotation: UiValue::Just(0.0).to_field().into(),
            direction: UiValue::Auto.into(),
            child_align: UiValue::Auto.into(),
            text: TextStyle::initial(),
            background: BackgroundInfo::default(),
        }
    }
}

pub struct ResCon {
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
