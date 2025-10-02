use crate::blanked_partial_ord;
use crate::color::RgbColor;
use crate::graphics::Drawable;
use crate::ui::elements::{Element, UiElement, UiElementStub};
use crate::ui::parse::parse_4xi32_abstract;
use crate::ui::res::MVR;
use crate::ui::styles::enums::{BackgroundRes, Geometry, Origin, TextAlign, TextFit};
use crate::ui::styles::interpolate::{BasicInterpolatable, Interpolator};
use crate::ui::styles::types::Dimension;
use crate::ui::styles::unit::Unit;
use crate::ui::styles::{InheritSupplier, Parseable, ResolveResult, UiStyle, UiValue, DEFAULT_STYLE};
use mvutils::unsafe_utils::DangerousCell;
use mvutils::utils::{PClamp, TetrahedronOp};
use std::rc::Rc;
use std::str::{FromStr, SplitWhitespace};

#[derive(Clone, Debug)]
pub struct TextStyle {
    pub size: UiValue<f32>,
    pub kerning: UiValue<f32>,
    pub skew: UiValue<f32>,
    pub stretch: UiValue<Dimension<f32>>,
    pub font: UiValue<usize>,
    pub fit: UiValue<TextFit>,
    pub color: UiValue<RgbColor>,
    pub select_color: UiValue<RgbColor>,
    pub align_x: UiValue<TextAlign>,
    pub align_y: UiValue<TextAlign>,
}

impl TextStyle {
    pub fn merge_unset(&mut self, other: &TextStyle) {
        self.size.merge_unset(&other.size);
        self.kerning.merge_unset(&other.kerning);
        self.skew.merge_unset(&other.skew);
        self.stretch.merge_unset(&other.stretch);
        self.font.merge_unset(&other.font);
        self.fit.merge_unset(&other.fit);
        self.color.merge_unset(&other.color);
        self.select_color.merge_unset(&other.select_color);
        self.align_y.merge_unset(&other.align_y);
        self.align_x.merge_unset(&other.align_x);
    }

    pub fn merge_at_set(&mut self, other: &TextStyle) {
        self.size.merge_at_set(&other.size);
        self.kerning.merge_at_set(&other.kerning);
        self.skew.merge_at_set(&other.skew);
        self.stretch.merge_at_set(&other.stretch);
        self.font.merge_at_set(&other.font);
        self.fit.merge_at_set(&other.fit);
        self.color.merge_at_set(&other.color);
        self.select_color.merge_at_set(&other.select_color);
        self.align_y.merge_at_set(&other.align_y);
        self.align_x.merge_at_set(&other.align_x);
    }
}

#[derive(Clone, Debug)]
pub struct TransformStyle {
    pub translate: VectorField<i32>,
    pub scale: VectorField<f32>,
    pub rotate: UiValue<f32>,
    pub origin: UiValue<Origin>,
}

impl TransformStyle {
    pub fn initial() -> Self {
        Self {
            translate: VectorField::splat(UiValue::Just(0)),
            scale: VectorField::splat(UiValue::Just(1.0)),
            rotate: UiValue::Just(0.0),
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

#[derive(Clone, Debug)]
pub struct VectorField<T: PartialOrd + Clone + 'static> {
    pub x: UiValue<T>,
    pub y: UiValue<T>,
}

impl<T: PartialOrd + Clone + 'static> VectorField<T> {
    pub fn splat(t: UiValue<T>) -> Self {
        Self { x: t.clone(), y: t }
    }

    pub fn set(&mut self, t: UiValue<T>) {
        self.x = t.clone();
        self.y = t;
    }

    pub fn resolve<F>(
        &self,
        dpi: f32,
        parent: Option<Element>,
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
        parent: Option<Element>,
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

#[derive(Clone, Debug)]
pub struct LayoutField<T: PartialOrd + Clone + 'static> {
    pub base: UiValue<T>,
    pub min: UiValue<T>,
    pub max: UiValue<T>,
}

impl<T: PartialOrd + Clone> LayoutField<T> {
    pub fn merge_at_set(&mut self, other: &Self) {
        self.base.merge_at_set(&other.base);
        self.min.merge_at_set(&other.min);
        self.max.merge_at_set(&other.max);
    }

    pub fn merge_unset(&mut self, other: &Self) {
        self.base.merge_unset(&other.base);
        self.min.merge_unset(&other.min);
        self.max.merge_unset(&other.max);
    }
    
    pub(crate) fn resolve<F, SF>(
        &self,
        dpi: f32,
        parent: Option<Element>,
        map: F,
        sup: &dyn InheritSupplier,
        sup_map: SF,
        self_value: T,
    ) -> ResolveResult<T>
    where
        F: Fn(&UiStyle) -> &Self,
        SF: Fn(&dyn InheritSupplier) -> T
    {
        let value = self.base.resolve(dpi, parent.clone(), |s| &map(s).base);
        let min = self.min.resolve(dpi, parent.clone(), |s| &map(s).min);
        let max = self.max.resolve(dpi, parent.clone(), |s| &map(s).max);

        if value.is_none() {
            return value;
        }

        let value = if value.is_auto() || value.is_use_default() {
            self_value
        } else {
            value.unwrap_or_default_or_percentage(&map(&DEFAULT_STYLE).base, parent.clone(), |s| sup_map(s), sup)
        };

        let mut emin = value.clone();
        let mut emax = value.clone();

        if !min.is_none() && !min.is_auto() && !min.is_use_default() {
            let u = min.unwrap_or_default_or_percentage(&map(&DEFAULT_STYLE).min, parent.clone(), |s| sup_map(s), sup);
            emin = u;
        }

        if !max.is_none() && !max.is_auto() && !max.is_use_default() {
            let u = max.unwrap_or_default_or_percentage(&map(&DEFAULT_STYLE).max, parent.clone(), |s| sup_map(s), sup);
            emax = u;
        }

        ResolveResult::Value(value.p_clamp(emin, emax))
    }
}

impl<T: PartialOrd + Clone> From<UiValue<T>> for LayoutField<T> {
    fn from(value: UiValue<T>) -> Self {
        LayoutField {
            base: value.clone(),
            min: value.clone(),
            max: value,
        }
    }
}

impl<T: PartialOrd + Clone> LayoutField<T> {
    pub fn is_set(&self) -> bool {
        self.base.is_set()
    }

    pub fn is_none(&self) -> bool {
        matches!(self.base, UiValue::None)
    }

    pub fn is_auto(&self) -> bool {
        matches!(self.base, UiValue::Auto)
    }

    pub fn is_unset(&self) -> bool {
        matches!(self.base, UiValue::Unset)
    }

    pub fn is_min_set(&self) -> bool {
        self.min.is_set()
    }

    pub fn is_min_none(&self) -> bool {
        matches!(self.min, UiValue::None)
    }

    pub fn is_min_auto(&self) -> bool {
        matches!(self.min, UiValue::Auto)
    }

    pub fn is_min_unset(&self) -> bool {
        matches!(self.min, UiValue::Unset)
    }

    pub fn is_max_set(&self) -> bool {
        self.max.is_set()
    }

    pub fn is_max_none(&self) -> bool {
        matches!(self.max, UiValue::None)
    }

    pub fn is_max_auto(&self) -> bool {
        matches!(self.max, UiValue::Auto)
    }

    pub fn is_max_unset(&self) -> bool {
        matches!(self.max, UiValue::Unset)
    }
}

impl<T: FromStr + Clone + PartialOrd + 'static> Parseable for LayoutField<T> {
    fn parse(s: &str) -> Result<Self, String> {
        let mut tokens = s.split_whitespace();

        let first = tokens.next().ok_or("expected base value")?;
        let base = UiValue::<T>::parse(first)?;
        let mut min = base.clone();
        let mut max = base.clone();

        let mut t_iter = tokens.peekable();
        while let Some(&kw) = t_iter.peek() {
            t_iter.next();
            let val_str = t_iter
                .next()
                .ok_or_else(|| format!("expected value after `{kw}`"))?;
            let val = UiValue::<T>::parse(val_str)?;

            match kw {
                "min" => min = val,
                "max" => max = val,
                _ => return Err(format!("unexpected keyword: {kw}")),
            }
        }

        Ok(LayoutField { base, min, max })
    }
}

impl Interpolator<TextStyle> for TextStyle {
    fn interpolate<E, F>(&mut self, start: &Self, end: &TextStyle, percent: f32, elem: &E, f: F)
    where
        E: UiElementStub,
        F: Fn(&UiStyle) -> &Self,
    {
        self.fit = (percent < 50f32).yn(start.fit.clone(), end.fit.clone());
        self.size
            .interpolate(&start.size, &end.size, percent, elem, |s| &f(s).size);
        self.font = (percent < 50f32).yn(start.font.clone(), end.font.clone());
        self.kerning
            .interpolate(&start.kerning, &end.kerning, percent, elem, |s| {
                &f(s).kerning
            });
        self.skew
            .interpolate(&start.skew, &end.skew, percent, elem, |s| &f(s).skew);
        self.stretch
            .interpolate(&start.stretch, &end.stretch, percent, elem, |s| {
                &f(s).stretch
            });
        self.color.interpolate(&start.color, &end.color, percent, elem, |s| &f(s).color);
        self.select_color.interpolate(&start.select_color, &end.select_color, percent, elem, |s| &f(s).select_color);
        self.align_x = (percent < 50f32).yn(start.align_x.clone(), end.align_x.clone());
        self.align_y = (percent < 50f32).yn(start.align_y.clone(), end.align_y.clone());
    }
}

#[derive(Clone, Debug)]
pub struct SideStyle {
    pub top: LayoutField<i32>,
    pub bottom: LayoutField<i32>,
    pub left: LayoutField<i32>,
    pub right: LayoutField<i32>,
}

impl SideStyle {
    pub fn all_i32(v: i32) -> Self {
        Self {
            top: UiValue::Just(v).to_field(),
            bottom: UiValue::Just(v).to_field(),
            left: UiValue::Just(v).to_field(),
            right: UiValue::Just(v).to_field(),
        }
    }

    pub fn all(v: UiValue<i32>) -> Self {
        Self {
            top: v.clone().to_field(),
            bottom: v.clone().to_field(),
            left: v.clone().to_field(),
            right: v.to_field(),
        }
    }

    pub fn inline(v: UiValue<i32>) -> Self {
        Self {
            top: UiValue::None.to_field(),
            bottom: UiValue::None.to_field(),
            left: v.clone().to_field(),
            right: v.to_field(),
        }
    }

    pub fn block(v: UiValue<i32>) -> Self {
        Self {
            left: UiValue::None.to_field(),
            right: UiValue::None.to_field(),
            top: v.clone().to_field(),
            bottom: v.to_field(),
        }
    }

    pub fn set(&mut self, v: UiValue<i32>) {
        self.top = v.clone().to_field();
        self.bottom = v.clone().to_field();
        self.left = v.clone().to_field();
        self.right = v.to_field();
    }

    pub fn get<E, F, PF>(
        &self,
        elem: &E,
        map: F,
        percent_map: PF,
        sup: &impl InheritSupplier,
    ) -> [i32; 4]
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
            }, sup, |s| percent_map(s)[0], 0);
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
            }, sup, |s| percent_map(s)[1], 0);
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
            }, sup, |s| percent_map(s)[2], 0);
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
            }, sup, |s| percent_map(s)[3], 0);
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
            top: a.to_field(),
            bottom: b.to_field(),
            left: c.to_field(),
            right: d.to_field(),
        })
    }
}

impl Interpolator<SideStyle> for SideStyle {
    fn interpolate<E, F>(&mut self, start: &Self, end: &Self, percent: f32, elem: &E, f: F)
    where
        E: UiElementStub,
        F: Fn(&UiStyle) -> &Self
    {
        self.left.interpolate(&start.left, &end.left, percent, elem, |s| &f(s).left);
        self.right.interpolate(&start.right, &end.right, percent, elem, |s| &f(s).right);
        self.top.interpolate(&start.top, &end.top, percent, elem, |s| &f(s).top);
        self.bottom.interpolate(&start.bottom, &end.bottom, percent, elem, |s| &f(s).bottom);
    }
}

#[derive(Clone, Debug)]
pub struct ShapeStyle {
    pub resource: UiValue<BasicInterpolatable<BackgroundRes>>,
    pub color: UiValue<RgbColor>,
    pub texture: UiValue<BasicInterpolatable<Drawable>>,
    pub shape: UiValue<BasicInterpolatable<Geometry>>,
    pub adaptive_ratio: UiValue<f32>,
}

impl ShapeStyle {
    pub fn initial() -> Self {
        Self {
            resource: UiValue::Just(BackgroundRes::Color.into()),
            color: UiValue::Just(RgbColor::white().into()),
            texture: UiValue::None,
            shape: UiValue::Just(BasicInterpolatable::new(Geometry::Shape(MVR.shape.rect))),
            adaptive_ratio: UiValue::Just(1.0),
        }
    }

    pub fn merge_unset(&mut self, other: &ShapeStyle) {
        self.resource.merge_unset(&other.resource);
        self.shape.merge_unset(&other.shape);
        self.color.merge_unset(&other.color);
        self.texture.merge_unset(&other.texture);
        self.adaptive_ratio.merge_unset(&other.adaptive_ratio);
    }

    pub fn merge_at_set(&mut self, other: &ShapeStyle) {
        self.resource.merge_at_set(&other.resource);
        self.shape.merge_at_set(&other.shape);
        self.color.merge_at_set(&other.color);
        self.texture.merge_at_set(&other.texture);
        self.adaptive_ratio.merge_at_set(&other.adaptive_ratio);
    }
}

impl Parseable for ShapeStyle {
    fn parse(s: &str) -> Result<Self, String> {
        let mut color = None;
        let mut texture = None;
        let mut shape = None;
        let mut ratio = None;

        fn next<T: Clone + FromStr + 'static>(i: &mut SplitWhitespace) -> Result<UiValue<T>, String> {
            let n = i.next().ok_or("Expected more tokens!".to_string())?;
            Parseable::parse(n)
        }

        macro_rules! all {
            ($v:expr) => {
                return Ok(ShapeStyle {
                    resource: $v,
                    color: $v,
                    texture: $v,
                    shape: $v,
                    adaptive_ratio: $v,
                })
            }
        }

        match s {
            "none" => all!(UiValue::None),
            "auto" => all!(UiValue::Auto),
            "unset" => all!(UiValue::Unset),
            "inherit" => all!(UiValue::Inherit),
            _ => {}
        }

        let mut tokens = s.split_whitespace();
        while let Some(token) = tokens.next() {
            match token {
                "color" => color = Some(next(&mut tokens)?),
                "texture" => texture = Some(next(&mut tokens)?),
                "shape" => shape = Some(next(&mut tokens)?),
                "ratio" => ratio = Some(next(&mut tokens)?),
                other => return Err(format!("Illegal ShapeStyle token: {other}")),
            }
        }
        Ok(ShapeStyle {
            resource: texture.is_some().yn(UiValue::Just(BackgroundRes::Texture.into()), UiValue::Auto),
            color: color.unwrap_or(UiValue::Unset),
            texture: texture.unwrap_or(UiValue::Unset),
            shape: shape.unwrap_or(UiValue::Unset),
            adaptive_ratio: ratio.unwrap_or(UiValue::Unset),
        })
    }
}

impl Interpolator<ShapeStyle> for ShapeStyle {
    fn interpolate<E, F>(&mut self, start: &Self, end: &Self, percent: f32, elem: &E, f: F)
    where
        E: UiElementStub,
        F: Fn(&UiStyle) -> &Self,
    {
        self.resource
            .interpolate(&start.resource, &end.resource, percent, elem, |s| {
                &f(s).resource
            });
        self.shape
            .interpolate(&start.shape, &end.shape, percent, elem, |s| {
                &f(s).shape
            });
        self.color
            .interpolate(&start.color, &end.color, percent, elem, |s| &f(s).color);
        self.texture
            .interpolate(&start.texture, &end.texture, percent, elem, |s| {
                &f(s).texture
            });
        self.adaptive_ratio
            .interpolate(&start.adaptive_ratio, &end.adaptive_ratio, percent, elem, |s| &f(s).adaptive_ratio);
    }
}

blanked_partial_ord!(ShapeStyle);

#[derive(Clone, Debug)]
pub struct ScrollBarStyle {
    pub track: ShapeStyle,
    pub knob: ShapeStyle,
    pub size: UiValue<i32>,
}

impl ScrollBarStyle {
    pub fn initial() -> Self {
        Self {
            track: ShapeStyle::initial(),
            knob: ShapeStyle::initial(),
            size: UiValue::Measurement(Unit::BeardFortnight(1.0)),
        }
    }

    pub fn merge_unset(&mut self, other: &ScrollBarStyle) {
        self.track.merge_unset(&other.track);
        self.knob.merge_unset(&other.knob);
        self.size.merge_unset(&other.size);
    }

    pub fn merge_at_set(&mut self, other: &ScrollBarStyle) {
        self.track.merge_at_set(&other.track);
        self.knob.merge_at_set(&other.knob);
        self.size.merge_at_set(&other.size);
    }
}
