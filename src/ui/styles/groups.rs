use std::rc::Rc;
use std::str::FromStr;
use mvutils::unsafe_utils::DangerousCell;
use mvutils::utils::{PClamp, TetrahedronOp};
use crate::color::RgbColor;
use crate::graphics::Drawable;
use crate::ui::elements::{UiElement, UiElementStub};
use crate::ui::parse::parse_4xi32_abstract;
use crate::ui::res::MVR;
use crate::ui::styles::{InheritSupplier, Resolve, ResolveResult, UiStyle, UiValue};
use crate::ui::styles::enums::{BackgroundRes, Origin, TextAlign, TextFit, UiShape};
use crate::ui::styles::interpolate::{BasicInterpolatable, Interpolator};
use crate::ui::styles::types::Dimension;
use crate::ui::styles::unit::Unit;

#[derive(Clone)]
pub struct TextStyle {
    pub size: Resolve<f32>,
    pub kerning: Resolve<f32>,
    pub skew: Resolve<f32>,
    pub stretch: Resolve<Dimension<f32>>,
    pub font: Resolve<usize>,
    pub fit: Resolve<TextFit>,
    pub color: Resolve<RgbColor>,
    pub align_x: Resolve<TextAlign>,
    pub align_y: Resolve<TextAlign>,
}

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
            align_x: UiValue::Auto.into(),
            align_y: UiValue::Auto.into(),
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

    pub fn merge_at_set(&mut self, other: &TextStyle) {
        self.size.merge_at_set(&other.size);
        self.kerning.merge_at_set(&other.kerning);
        self.skew.merge_at_set(&other.skew);
        self.stretch.merge_at_set(&other.stretch);
        self.font.merge_at_set(&other.font);
        self.fit.merge_at_set(&other.fit);
        self.color.merge_at_set(&other.color);
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

    pub fn merge_at_set(&mut self, other: &TransformStyle) {
        self.translate.merge_at_set(&other.translate);
        self.scale.merge_at_set(&other.scale);
        self.rotate.merge_at_set(&other.rotate);
        self.origin.merge_at_set(&other.origin);
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
        parent: Option<Rc<DangerousCell<UiElement>>>,
        map: F,
    ) -> (ResolveResult<T>, ResolveResult<T>)
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
        parent: Option<Rc<DangerousCell<UiElement>>>,
        map: F,
        def: (T, T),
    ) -> (T, T)
    where
        F: Fn(&UiStyle) -> &Self,
    {
        let x_res = self.x.resolve(dpi, parent.clone(), |s| &map(s).x);
        let y_res = self.y.resolve(dpi, parent, |s| &map(s).y);

        let mut res = def;
        if x_res.is_set() {
            res.0 = x_res.unwrap()
        }
        if y_res.is_set() {
            res.1 = y_res.unwrap()
        }
        res
    }

    pub fn merge_unset(&mut self, other: &VectorField<T>) {
        self.x.merge_unset(&other.x);
        self.y.merge_unset(&other.y);
    }

    pub fn merge_at_set(&mut self, other: &VectorField<T>) {
        self.x.merge_at_set(&other.x);
        self.y.merge_at_set(&other.y);
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

    pub(crate) fn resolve<F>(
        &self,
        dpi: f32,
        parent: Option<Rc<DangerousCell<UiElement>>>,
        map: F,
    ) -> ResolveResult<T>
    where
        F: Fn(&UiStyle) -> &Self,
    {
        let value = self.value.resolve(dpi, parent.clone(), |s| &map(s).value);
        let min = self.min.resolve(dpi, parent.clone(), |s| &map(s).min);
        let max = self.max.resolve(dpi, parent, |s| &map(s).max);

        if !value.is_set() {
            return value;
        }

        let emin;
        let emax;

        if min.is_set() {
            emin = Some(min.unwrap());
        } else {
            emin = Some(value.clone().unwrap());
        }

        if max.is_set() {
            emax = Some(max.unwrap());
        } else {
            emax = Some(value.clone().unwrap());
        }

        ResolveResult::Value(value.unwrap().p_clamp(emin.unwrap(), emax.unwrap()))
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

        if min.is_set() {
            let min = min.unwrap();
            if ret < min {
                ret = min;
            }
        }

        if max.is_set() {
            let max = max.unwrap();
            if ret > max {
                ret = max;
            }
        }

        ret
    }
}

impl Interpolator<TextStyle> for TextStyle {
    fn interpolate<E, F>(&mut self, start: &Self, end: &TextStyle, percent: f32, elem: &E, _: F)
    where
        E: UiElementStub,
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

#[derive(Clone)]
pub struct SideStyle {
    pub top: Resolve<i32>,
    pub bottom: Resolve<i32>,
    pub left: Resolve<i32>,
    pub right: Resolve<i32>,
}

impl SideStyle {
    // this is just so the proc macro doesnt fuck itself up
    // band aid on leaky pipe type shit
    pub fn for_value<F>(&mut self, f: F)
    where
        F: Fn(&mut SideStyle),
    {
        f(self);
        
    }
    
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

    pub fn inline(v: Resolve<i32>) -> Self {
        Self {
            top: UiValue::None.to_resolve(),
            bottom: UiValue::None.to_resolve(),
            left: v.clone(),
            right: v,
        }
    }

    pub fn block(v: Resolve<i32>) -> Self {
        Self {
            left: UiValue::None.to_resolve(),
            right: UiValue::None.to_resolve(),
            top: v.clone(),
            bottom: v,
        }
    }

    pub fn set(&mut self, v: Resolve<i32>) {
        self.top = v.clone();
        self.bottom = v.clone();
        self.left = v.clone();
        self.right = v;
    }

    pub fn get<E, F, PF>(&self, elem: &E, map: F, percent_map: PF, sup: &impl InheritSupplier) -> [i32; 4]
    where
        E: UiElementStub,
        F: Fn(&UiStyle) -> &Self,
        PF: Fn(&dyn InheritSupplier) -> [i32; 4], //t, b, l, r
    {
        let parent = elem.state().parent.clone();

        let top = self
            .top
            .resolve(elem.state().ctx.dpi, elem.state().parent.clone(), |s| {
                &map(s).top
            });
        let top = if top.is_set() {
            top.unwrap()
        } else if top.is_percent() {
            top.resolve_percent(parent.clone(), |s| percent_map(s)[0], sup)
        } else {
            self.top.is_auto().yn(5, 0)
        };

        let bottom = self
            .bottom
            .resolve(elem.state().ctx.dpi, elem.state().parent.clone(), |s| {
                &map(s).bottom
            });
        let bottom = if bottom.is_set() {
            bottom.unwrap()
        } else if bottom.is_percent() {
            bottom.resolve_percent(parent.clone(), |s| percent_map(s)[1], sup)
        } else {
            self.bottom.is_auto().yn(5, 0)
        };

        let left = self
            .left
            .resolve(elem.state().ctx.dpi, elem.state().parent.clone(), |s| {
                &map(s).left
            });
        let left = if left.is_set() {
            left.unwrap()
        } else if left.is_percent() {
            left.resolve_percent(parent.clone(), |s| percent_map(s)[2], sup)
        } else {
            self.left.is_auto().yn(5, 0)
        };

        let right = self
            .right
            .resolve(elem.state().ctx.dpi, elem.state().parent.clone(), |s| {
                &map(s).right
            });
        let right = if right.is_set() {
            right.unwrap()
        } else if right.is_percent() {
            right.resolve_percent(parent.clone(), |s| percent_map(s)[3], sup)
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

    pub fn merge_at_set(&mut self, other: &SideStyle) {
        self.bottom.merge_at_set(&other.bottom);
        self.top.merge_at_set(&other.top);
        self.left.merge_at_set(&other.left);
        self.right.merge_at_set(&other.right);
    }
}

impl FromStr for SideStyle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = parse_4xi32_abstract(s).map_err(|_| "Whoops!?! Looks like your amazing string wasn't compatible with our patented parse_4xi32 function...".to_string())?;
        let [a, b, c, d] = s;
        Ok(Self {
            top: a,
            bottom: b,
            left: c,
            right: d,
        })
    }
}

#[derive(Clone)]
pub struct ShapeStyle {
    pub resource: Resolve<BasicInterpolatable<BackgroundRes>>,
    pub color: Resolve<RgbColor>,
    pub texture: Resolve<BasicInterpolatable<Drawable>>,
    pub shape: Resolve<BasicInterpolatable<UiShape>>,
}

impl ShapeStyle {
    pub fn initial() -> Self {
        Self {
            resource: UiValue::Just(BackgroundRes::Color.into()).to_resolve(),
            color: UiValue::Just(RgbColor::white().into()).to_resolve(),
            texture: UiValue::None.to_resolve(),
            shape: UiValue::Just(BasicInterpolatable::new(UiShape::Shape(MVR.shape.rect)))
                .to_resolve(),
        }
    }

    pub fn merge_unset(&mut self, other: &ShapeStyle) {
        self.resource.merge_unset(&other.resource);
        self.color.merge_unset(&other.color);
        self.texture.merge_unset(&other.texture);
    }

    pub fn merge_at_set(&mut self, other: &ShapeStyle) {
        self.resource.merge_at_set(&other.resource);
        self.color.merge_at_set(&other.color);
        self.texture.merge_at_set(&other.texture);
    }
}