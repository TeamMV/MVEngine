use crate::blanked_partial_ord;
use crate::elements::{UiElement, UiElementState, UiElementStub};
use mvcore::color::{Color, ColorFormat, RgbColor};
use mvutils::{enum_val_ref, lazy};
use mvutils::save::Savable;
use mvutils::unsafe_utils::Unsafe;
use mvutils::utils::{PClamp, TetrahedronOp};
use num_traits::Num;
use parking_lot::RwLock;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::Add;
use std::sync::Arc;

lazy! {
    pub static DEFAULT_STYLE: UiStyle = UiStyle {
        x: UiValue::None.to_field().to_resolve(),
        y: UiValue::None.to_field().to_resolve(),
        width: UiValue::Auto.to_field().to_resolve(),
        height: UiValue::Auto.to_field().to_resolve(),
        padding: SideStyle::all(UiValue::Measurement(Unit::BeardFortnight(1.0)).to_field().to_resolve()),
        margin: SideStyle::all(UiValue::Measurement(Unit::BeardFortnight(1.0)).to_field().to_resolve()),
        origin: UiValue::Just(Origin::BottomLeft).to_resolve(),
        position: UiValue::Just(Position::Relative).to_resolve(),
        direction: UiValue::Just(Direction::Horizontal).to_resolve(),
        child_align: UiValue::Just(ChildAlign::Start).to_resolve(),
        text: TextStyle {
            size: UiValue::Measurement(Unit::Line(1.0)).to_field().to_resolve(),
            kerning: UiValue::None.to_field().to_resolve(),
            skew: UiValue::None.to_field().to_resolve(),
            stretch: UiValue::None.to_field().to_resolve(),
            font: UiValue::Just(0).to_field().to_resolve(),
            fit: UiValue::Just(TextFit::ExpandParent).to_field().to_resolve(),
            color: UiValue::Just(RgbColor::black()).to_resolve(),
        },
        transform: TransformStyle {
            translate: VectorField::splat(UiValue::Just(0).to_field().to_resolve()),
            scale: VectorField::splat(UiValue::Just(1.0).to_field().to_resolve()),
            rotate: UiValue::Just(0.0).to_field().to_resolve(),
            origin: UiValue::Just(Origin::Center).to_field().to_resolve(),
        },
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
        child_align: UiValue::Unset.to_resolve(),
        text: TextStyle {
            size: UiValue::Unset.to_field().to_resolve(),
            kerning: UiValue::Unset.to_field().to_resolve(),
            skew: UiValue::Unset.to_field().to_resolve(),
            stretch: UiValue::Unset.to_field().to_resolve(),
            font: UiValue::Unset.to_field().to_resolve(),
            fit: UiValue::Unset.to_field().to_resolve(),
            color: UiValue::Unset.to_resolve(),
        },
        transform: TransformStyle {
            translate: VectorField::splat(UiValue::Unset.to_field().to_resolve()),
            scale: VectorField::splat(UiValue::Unset.to_field().to_resolve()),
            rotate: UiValue::Unset.to_field().to_resolve(),
            origin: UiValue::Unset.to_field().to_resolve(),
        },
    };
}

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
                $crate::styles::DEFAULT_STYLE.$($style).*
                .resolve($elem.state().ctx.dpi, None, |s| {&s.$($style).*})
                .expect("Default style could not be resolved")
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
    pub child_align: Resolve<ChildAlign>,

    pub text: TextStyle,
    pub transform: TransformStyle,
}

unsafe impl Sync for UiStyle {}
unsafe impl Send for UiStyle {}

impl UiStyle {
    pub fn merge_unset(&mut self, other: &UiStyle) {
        self.x.merge_unset(&other.x);
        self.y.merge_unset(&other.y);
        self.width.merge_unset(&other.height);
        self.padding.merge_unset(&other.padding);
        self.margin.merge_unset(&other.margin);
        self.origin.merge_unset(&other.origin);
        self.position.merge_unset(&other.position);
        self.direction.merge_unset(&other.direction);
        self.child_align.merge_unset(&other.child_align);
        self.text.merge_unset(&other.text);
        self.transform.merge_unset(&other.transform);
    }
}

blanked_partial_ord!(UiStyle);

#[derive(Default, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Origin {
    TopLeft,
    #[default]
    BottomLeft,
    TopRight,
    BottomRight,
    Center,
    Custom(i32, i32),
    Eval(fn(i32, i32, i32, i32) -> (i32, i32)),
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

    pub fn get_actual_x(&self, x: i32, width: i32, state: &UiElementState) -> i32 {
        match self {
            Origin::TopLeft => x,
            Origin::BottomLeft => x,
            Origin::TopRight => x - width,
            Origin::BottomRight => x - width,
            Origin::Center => x - width / 2,
            Origin::Custom(cx, _) => x - cx,
            Origin::Eval(f) => {
                let res = f(
                    state.bounding_rect.x,
                    state.bounding_rect.y,
                    state.bounding_rect.width,
                    state.bounding_rect.height,
                );
                x - res.0
            }
        }
    }

    pub fn get_actual_y(&self, y: i32, height: i32, state: &UiElementState) -> i32 {
        match self {
            Origin::TopLeft => y - height,
            Origin::BottomLeft => y,
            Origin::TopRight => y - height,
            Origin::BottomRight => y,
            Origin::Center => y - height / 2,
            Origin::Custom(_, cy) => y - cy,
            Origin::Eval(f) => {
                let res = f(
                    state.bounding_rect.x,
                    state.bounding_rect.y,
                    state.bounding_rect.width,
                    state.bounding_rect.height,
                );
                y - res.1
            }
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
    pub font: Resolve<u16>,
    pub fit: Resolve<TextFit>,
    pub color: Resolve<RgbColor>,
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

    pub fn merge_unset(&mut self, other: &TextStyle) {
        self.size.merge_unset(&other.size);
        self.kerning.merge_unset(&other.kerning);
        self.skew.merge_unset(&other.skew);
        self.stretch.merge_unset(&other.stretch);
        self.font.merge_unset(&other.font);
        self.fit.merge_unset(&other.fit);
        self.color.merge_unset(&other.color);
    }
}

#[derive(Clone)]
pub struct TransformStyle {
    pub translate: VectorField<i32>,
    pub scale: VectorField<f32>,
    pub rotate: Resolve<f32>,
    pub origin: Resolve<Origin>,
}

impl TransformStyle {
    pub fn initial() -> Self {
        Self {
            translate: VectorField::splat(UiValue::Just(0).to_field().into()),
            scale: VectorField::splat(UiValue::Just(1.0).to_field().into()),
            rotate: UiValue::Just(0.0).to_field().into(),
            origin: UiValue::Just(Origin::Center).into(),
        }
    }

    pub fn merge_unset(&mut self, other: &TransformStyle) {
        self.translate.merge_unset(&other.translate);
        self.scale.merge_unset(&other.scale);
        self.rotate.merge_unset(&other.rotate);
        self.origin.merge_unset(&other.origin);
    }
}

#[derive(Clone)]
pub struct VectorField<T: PartialOrd + Clone + 'static> {
    pub x: Resolve<T>,
    pub y: Resolve<T>,
}

impl<T: PartialOrd + Clone + 'static> VectorField<T> {
    pub fn splat(t: Resolve<T>) -> Self {
        Self { x: t.clone(), y: t }
    }

    pub fn set(&mut self, t: Resolve<T>) {
        self.x = t.clone();
        self.y = t;
    }

    pub fn resolve<F>(
        &self,
        dpi: f32,
        parent: Option<Arc<RwLock<UiElement>>>,
        map: F,
    ) -> (Option<T>, Option<T>)
    where
        F: Fn(&UiStyle) -> &Self,
    {
        let x_res = self.x.resolve(dpi, parent.clone(), |s| &map(s).x);
        let y_res = self.y.resolve(dpi, parent, |s| &map(s).y);
        (x_res, y_res)
    }

    pub fn resolve_with_default<F>(
        &self,
        dpi: f32,
        parent: Option<Arc<RwLock<UiElement>>>,
        map: F,
        def: (T, T),
    ) -> (T, T)
    where
        F: Fn(&UiStyle) -> &Self,
    {
        let x_res = self.x.resolve(dpi, parent.clone(), |s| &map(s).x);
        let y_res = self.y.resolve(dpi, parent, |s| &map(s).y);

        let mut res = def;
        if x_res.is_some() {
            res.0 = x_res.unwrap()
        }
        if y_res.is_some() {
            res.1 = y_res.unwrap()
        }
        res
    }

    pub fn merge_unset(&mut self, other: &VectorField<T>) {
        self.x.merge_unset(&other.x);
        self.y.merge_unset(&other.y);
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

    fn resolve<F>(&self, dpi: f32, parent: Option<Arc<RwLock<UiElement>>>, map: F) -> Option<T>
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

    pub fn is_unset(&self) -> bool {
        return matches!(self.value, UiValue::Unset);
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

    pub fn is_min_unset(&self) -> bool {
        return matches!(self.min, UiValue::Unset);
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

    pub fn is_max_unset(&self) -> bool {
        return matches!(self.max, UiValue::Unset);
    }

    pub fn apply<F>(&self, value: T, elem: &UiElement, map: F) -> T
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
    pub point: Point<T>,
    pub dimension: Dimension<T>,
    pub rotation: f32,
    pub origin: Point<T>,
}

impl<T: Num + Clone + Debug> Location<T> {
    pub fn new(point: Point<T>, dimension: Dimension<T>, rotation: f32, origin: Point<T>) -> Self {
        Self {
            point,
            dimension,
            rotation,
            origin,
        }
    }

    pub fn simple(point: Point<T>, dimension: Dimension<T>) -> Self {
        let (w, h) = dimension.components();
        Self {
            point,
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
            top: UiValue::Just(v).to_field().to_resolve(),
            bottom: UiValue::Just(v).to_field().to_resolve(),
            left: UiValue::Just(v).to_field().to_resolve(),
            right: UiValue::Just(v).to_field().to_resolve(),
        }
    }

    pub fn all(v: Resolve<i32>) -> Self {
        Self {
            top: v.clone(),
            bottom: v.clone(),
            left: v.clone(),
            right: v,
        }
    }

    pub fn set(&mut self, v: Resolve<i32>) {
        self.top = v.clone();
        self.bottom = v.clone();
        self.left = v.clone();
        self.right = v;
    }

    pub fn get<F>(&self, elem: &UiElement, map: F) -> [i32; 4]
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

    pub fn merge_unset(&mut self, other: &SideStyle) {
        self.bottom.merge_unset(&other.bottom);
        self.top.merge_unset(&other.top);
        self.left.merge_unset(&other.left);
        self.right.merge_unset(&other.right);
    }
}

#[derive(Clone)]
pub enum Resolve<T: PartialOrd + Clone + 'static> {
    UiValue(UiValue<T>),
    LayoutField(LayoutField<T>),
}

impl<T: PartialOrd + Clone + 'static> Resolve<T> {
    pub fn resolve<F>(&self, dpi: f32, parent: Option<Arc<RwLock<UiElement>>>, map: F) -> Option<T>
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
    Clone(Arc<UiElement>),
    Just(T),
    Measurement(Unit),
}

impl<T: Clone + PartialOrd + 'static> UiValue<T> {
    pub fn to_field(self) -> LayoutField<T> {
        LayoutField::from(self)
    }

    pub fn to_resolve(self) -> Resolve<T> {
        Resolve::UiValue(self)
    }

    pub fn resolve<F>(&self, dpi: f32, parent: Option<Arc<RwLock<UiElement>>>, map: F) -> Option<T>
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
            UiValue::Unset => {
                unsafe {
                    if parent.is_some() {
                        let cloned = parent.clone().unwrap();
                        let p_guard = cloned.read();
                        let parent_value = Unsafe::cast_static(map(p_guard.style()));
                        drop(p_guard);
                        return parent_value.resolve(dpi, parent.clone(), map);
                    }
                }
                None
            }
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

fn no_parent<T>() -> T {
    panic!("Called Inherit on UiElement without parent")
}

pub trait Interpolator<T: PartialOrd + Clone + 'static> {
    fn interpolate<F>(&mut self, start: &Self, end: &Self, percent: f32, elem: &UiElement, f: F)
    where
        F: Fn(&UiStyle) -> &Self;
}

macro_rules! impl_interpolator_primitives {
    ($($t:ty,)*) => {
        $(
            impl Interpolator<$t> for $t {
                fn interpolate<F>(&mut self, start: &$t, end: &$t, percent: f32, elem: &UiElement, f: F)
                    where
                        F: Fn(&UiStyle) -> &Self,
                {
                    let frac = percent / 100.0;

                    *self = (*start as f32 + ((*end as f32 - *start as f32) * frac)) as $t;
                }
            }
        )*
    };
}

impl_interpolator_primitives!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64,);

impl<T: PartialOrd + Clone + 'static + Interpolator<T>> Interpolator<T> for Resolve<T> {
    fn interpolate<F>(&mut self, start: &Self, end: &Self, percent: f32, elem: &UiElement, f: F)
    where
        F: Fn(&UiStyle) -> &Self,
    {
        let state = elem.state();

        let mut self_resolve = self.resolve(state.ctx.dpi, state.parent.clone(), |s| f(s));
        let start_resolve = start.resolve(state.ctx.dpi, state.parent.clone(), |s| f(s));
        let end_resolve = end.resolve(state.ctx.dpi, state.parent.clone(), |s| f(s));

        if start_resolve.is_none() || end_resolve.is_none() || self_resolve.is_none() {
            return;
        }

        let start_resolve = start_resolve.unwrap();
        let mut self_resolve = self_resolve.unwrap();
        let end_resolve = end_resolve.unwrap();

        let dpi = state.ctx.dpi;
        let parent = state.parent.clone();

        //why do i have to clone parent again???
        unsafe {
            self_resolve.interpolate(&start_resolve, &end_resolve, percent, elem, move |s| {
                Unsafe::cast_static(&f(s).resolve(dpi, parent.clone(), |s2| &f(s2)).unwrap())
            });

            *self = Resolve::UiValue(UiValue::Just(self_resolve));
        }
    }
}

impl Interpolator<UiStyle> for UiStyle {
    fn interpolate<F>(&mut self, start: &Self, end: &UiStyle, percent: f32, elem: &UiElement, f: F)
    where
        F: Fn(&UiStyle) -> &Self,
    {
        self.x
            .interpolate(&start.x, &end.x, percent, elem, |s| &s.x);
        self.y
            .interpolate(&start.y, &end.y, percent, elem, |s| &s.y);
        self.width
            .interpolate(&start.width, &end.width, percent, elem, |s| &s.width);
        self.height
            .interpolate(&start.height, &end.height, percent, elem, |s| &s.height);
        self.padding
            .left
            .interpolate(&start.padding.left, &end.padding.left, percent, elem, |s| {
                &s.padding.left
            });
        self.padding.right.interpolate(
            &start.padding.right,
            &end.padding.right,
            percent,
            elem,
            |s| &s.padding.right,
        );
        self.padding
            .top
            .interpolate(&start.padding.top, &end.padding.top, percent, elem, |s| {
                &s.padding.top
            });
        self.padding.bottom.interpolate(
            &start.padding.bottom,
            &end.padding.bottom,
            percent,
            elem,
            |s| &s.padding.bottom,
        );

        self.margin
            .left
            .interpolate(&start.margin.left, &end.margin.left, percent, elem, |s| {
                &s.margin.left
            });
        self.margin
            .right
            .interpolate(&start.margin.right, &end.margin.right, percent, elem, |s| {
                &s.margin.right
            });
        self.margin
            .top
            .interpolate(&start.margin.top, &end.margin.top, percent, elem, |s| {
                &s.margin.top
            });
        self.margin.bottom.interpolate(
            &start.margin.bottom,
            &end.margin.bottom,
            percent,
            elem,
            |s| &s.margin.bottom,
        );

        self.origin = (percent < 50f32).yn(start.origin.clone(), end.origin.clone());
        self.position = (percent < 50f32).yn(start.position.clone(), end.position.clone());
        self.direction = (percent < 50f32).yn(start.direction.clone(), end.direction.clone());
        self.child_align = (percent < 50f32).yn(start.child_align.clone(), end.child_align.clone());

        self.text
            .size
            .interpolate(&start.text.size, &end.text.size, percent, elem, |s| {
                &s.text.size
            });
        self.text
            .stretch
            .interpolate(&start.text.stretch, &end.text.stretch, percent, elem, |s| {
                &s.text.stretch
            });
        self.text
            .skew
            .interpolate(&start.text.skew, &end.text.skew, percent, elem, |s| {
                &s.text.skew
            });
        self.text
            .kerning
            .interpolate(&start.text.kerning, &end.text.kerning, percent, elem, |s| {
                &s.text.kerning
            });
        self.text
            .color
            .interpolate(&start.text.color, &end.text.color, percent, elem, |s| {
                &s.text.color
            });
        self.text.fit = (percent < 50f32).yn(start.text.fit.clone(), end.text.fit.clone());
        self.text.font = (percent < 50f32).yn(start.text.font.clone(), end.text.font.clone());

        self.transform.translate.x.interpolate(
            &start.transform.translate.x,
            &end.transform.translate.x,
            percent,
            elem,
            |s| &f(s).transform.translate.x,
        );
        self.transform.translate.y.interpolate(
            &start.transform.translate.y,
            &end.transform.translate.y,
            percent,
            elem,
            |s| &f(s).transform.translate.y,
        );
        self.transform.scale.x.interpolate(
            &start.transform.scale.x,
            &end.transform.scale.x,
            percent,
            elem,
            |s| &f(s).transform.scale.x,
        );
        self.transform.scale.y.interpolate(
            &start.transform.scale.y,
            &end.transform.scale.y,
            percent,
            elem,
            |s| &f(s).transform.scale.y,
        );
        self.transform.rotate.interpolate(
            &start.transform.rotate,
            &end.transform.rotate,
            percent,
            elem,
            |s| &f(s).transform.rotate,
        );
        self.transform.origin =
            (percent > 50.0).yn(end.transform.origin.clone(), start.transform.origin.clone());
    }
}

impl Interpolator<TextStyle> for TextStyle {
    fn interpolate<F>(
        &mut self,
        start: &Self,
        end: &TextStyle,
        percent: f32,
        elem: &UiElement,
        f: F,
    ) where
        F: Fn(&UiStyle) -> &Self,
    {
        self.fit = (percent < 50f32).yn(self.fit.clone(), end.fit.clone());
        self.size
            .interpolate(&start.size, &end.size, percent, elem, |s| &s.text.size);
        self.font = (percent < 50f32).yn(self.font.clone(), end.font.clone());
        self.kerning
            .interpolate(&start.kerning, &end.kerning, percent, elem, |s| {
                &s.text.kerning
            });
        self.skew
            .interpolate(&start.skew, &end.skew, percent, elem, |s| &s.text.skew);
        self.stretch
            .interpolate(&start.stretch, &end.stretch, percent, elem, |s| {
                &s.text.stretch
            });
    }
}

impl<T: Interpolator<T> + Num + Clone + Debug + PartialOrd + 'static> Interpolator<Dimension<T>>
    for Dimension<T>
{
    fn interpolate<F>(
        &mut self,
        start: &Self,
        end: &Dimension<T>,
        percent: f32,
        elem: &UiElement,
        f: F,
    ) where
        F: Fn(&UiStyle) -> &Self,
    {
        self.width
            .interpolate(&start.width, &end.width, percent, elem, |s| &f(s).width);
        self.height
            .interpolate(&start.height, &end.height, percent, elem, |s| &f(s).height);
    }
}

impl<T: Interpolator<T> + Num + Clone + Debug + PartialOrd + 'static> Interpolator<Point<T>>
    for Point<T>
{
    fn interpolate<F>(&mut self, start: &Self, end: &Point<T>, percent: f32, elem: &UiElement, f: F)
    where
        F: Fn(&UiStyle) -> &Self,
    {
        self.x
            .interpolate(&start.x, &end.x, percent, elem, |s| &f(s).x);
        self.y
            .interpolate(&start.y, &end.y, percent, elem, |s| &f(s).y);
    }
}

impl<Fmt: ColorFormat + 'static> Interpolator<Color<Fmt>> for Color<Fmt>
where
    Fmt::ComponentType: Interpolator<Fmt::ComponentType> + PartialOrd<Fmt::ComponentType>,
{
    fn interpolate<F>(&mut self, start: &Self, end: &Self, percent: f32, elem: &UiElement, f: F)
    where
        F: Fn(&UiStyle) -> &Self,
    {
        let comp = self.components_mut();
        let start_comp = start.components();
        let end_comp = end.components();
        for i in 0..comp.len() {
            comp[i].interpolate(&start_comp[i], &end_comp[i], percent, elem, |s| {
                &f(s).components()[i]
            });
        }
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
            direction: UiValue::Auto.into(),
            child_align: UiValue::Auto.into(),
            text: TextStyle::initial(),
            transform: TransformStyle::initial(),
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
    Px(i32),             // px
    MM(f32),             // mm
    CM(f32),             // cm
    M(f32),              // m
    In(f32),             // in
    Twip(f32),           // twip
    Mil(f32),            // mil
    Point(f32),          // pt
    Pica(f32),           // pica
    Foot(f32),           // ft
    Yard(f32),           // yd
    Link(f32),           // lk
    Rod(f32),            // rd
    Chain(f32),          // ch
    Line(f32),           // ln
    BarleyCorn(f32),     // bc
    Nail(f32),           // nl
    Finger(f32),         // fg
    Stick(f32),          // sk
    Palm(f32),           // pm
    Shaftment(f32),      // sf
    Span(f32),           // sp
    Quarter(f32),        // qr
    Pace(f32),           // pc
    BeardFortnight(f32), //bf
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
            Unit::Line(value) => ((value * (1.0 / 40.0)) * dpi) as i32,
            Unit::BarleyCorn(value) => ((value * 0.125) * dpi) as i32,
            Unit::Nail(value) => ((value * 0.25) * dpi) as i32,
            Unit::Finger(value) => ((value * 0.375) * dpi) as i32,
            Unit::Stick(value) => ((value * 0.5) * dpi) as i32,
            Unit::Palm(value) => ((value * 3.0) * dpi) as i32,
            Unit::Shaftment(value) => ((value * 6.0) * dpi) as i32,
            Unit::Span(value) => ((value * 9.0) * dpi) as i32,
            Unit::Quarter(value) => ((value * 36.0) * dpi) as i32,
            Unit::Pace(value) => ((value * 30.0) * dpi) as i32,
            Unit::BeardFortnight(value) => ((value * 0.6048 * 0.393701) * dpi) as i32,
        }
    }
}