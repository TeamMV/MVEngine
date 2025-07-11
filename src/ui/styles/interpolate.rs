use crate::color::{Color, ColorFormat};
use crate::ui::elements::UiElementStub;
use crate::ui::styles::{Resolve, UiStyle, UiValue, DEFAULT_STYLE};
use mvutils::unsafe_utils::Unsafe;
use mvutils::utils::TetrahedronOp;
use std::cmp::Ordering;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use crate::resolve;
use crate::ui::styles::enums::TextAlign;

pub trait Interpolator<T: PartialOrd + Clone + 'static> {
    fn interpolate<E, F>(&mut self, start: &Self, end: &Self, percent: f32, elem: &E, f: F)
    where
        E: UiElementStub,
        F: Fn(&UiStyle) -> &Self;
}

#[derive(Clone, Debug)]
pub struct BasicInterpolatable<T: Clone + 'static> {
    t: T,
}

impl<T: Clone> PartialEq<Self> for BasicInterpolatable<T> {
    fn eq(&self, _: &Self) -> bool {
        false // Like you would never ever use this, it is just required ._.
    }
}

impl<T: Clone> PartialOrd for BasicInterpolatable<T> {
    fn partial_cmp(&self, _: &Self) -> Option<Ordering> {
        None
    }
}

impl<T: Clone + 'static> From<T> for BasicInterpolatable<T> {
    fn from(value: T) -> Self {
        BasicInterpolatable { t: value }
    }
}

impl<T: Clone + 'static> BasicInterpolatable<T> {
    pub const fn new(t: T) -> Self {
        Self { t }
    }
}

impl<T: Clone + 'static> Interpolator<BasicInterpolatable<T>> for BasicInterpolatable<T> {
    fn interpolate<E, F>(&mut self, start: &Self, end: &Self, percent: f32, _: &E, _: F)
    where
        E: UiElementStub,
        F: Fn(&UiStyle) -> &Self,
    {
        if percent > 50.0 {
            end.clone_into(self);
        } else {
            start.clone_into(self);
        }
    }
}

impl<T: Clone> Deref for BasicInterpolatable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.t
    }
}

impl<T: Clone> DerefMut for BasicInterpolatable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.t
    }
}

impl<T: PartialOrd + Clone + 'static + Interpolator<T>> Interpolator<T> for Resolve<T> {
    fn interpolate<E, F>(&mut self, start: &Self, end: &Self, percent: f32, elem: &E, f: F)
    where
        E: UiElementStub,
        F: Fn(&UiStyle) -> &Self,
    {
        let state = elem.state();

        let self_resolve = self.resolve(state.ctx.dpi, state.parent.clone(), |s| f(s));
        let start_resolve = start.resolve(state.ctx.dpi, state.parent.clone(), |s| f(s));
        let end_resolve = end.resolve(state.ctx.dpi, state.parent.clone(), |s| f(s));

        if !start_resolve.is_set() || !end_resolve.is_set() || !self_resolve.is_set() {
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
                Unsafe::cast_lifetime(&f(s).resolve(dpi, parent.clone(), |s2| &f(s2)).unwrap())
            });

            *self = Resolve::UiValue(UiValue::Just(self_resolve));
        }
    }
}

impl<T: FromStr + Clone> FromStr for BasicInterpolatable<T> {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        T::from_str(s)
            .map(|t| BasicInterpolatable::from(t))
            .map_err(|_| format!("{s} cannot be parsed!"))
    }
}

impl Interpolator<UiStyle> for UiStyle {
    fn interpolate<E, F>(&mut self, start: &Self, end: &UiStyle, percent: f32, elem: &E, f: F)
    where
        E: UiElementStub,
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
        self.child_align_x =
            (percent < 50f32).yn(start.child_align_x.clone(), end.child_align_x.clone());

        self.background.resource.interpolate(
            &start.background.resource,
            &end.background.resource,
            percent,
            elem,
            |s| &s.background.resource,
        );
        self.background.color.interpolate(
            &start.background.color,
            &end.background.color,
            percent,
            elem,
            |s| &s.background.color,
        );
        self.background.texture.interpolate(
            &start.background.texture,
            &end.background.texture,
            percent,
            elem,
            |s| &s.background.texture,
        );
        self.background.shape.interpolate(
            &start.background.shape,
            &end.background.shape,
            percent,
            elem,
            |s| &s.background.shape,
        );

        self.border.resource.interpolate(
            &start.border.resource,
            &end.border.resource,
            percent,
            elem,
            |s| &s.border.resource,
        );
        self.border
            .color
            .interpolate(&start.border.color, &end.border.color, percent, elem, |s| {
                &s.border.color
            });
        self.border.texture.interpolate(
            &start.border.texture,
            &end.border.texture,
            percent,
            elem,
            |s| &s.border.texture,
        );
        self.border.shape.interpolate(
            &start.border.shape,
            &end.border.shape,
            percent,
            elem,
            |s| &s.border.shape,
        );

        self.detail.resource.interpolate(
            &start.detail.resource,
            &end.detail.resource,
            percent,
            elem,
            |s| &s.detail.resource,
        );
        self.detail
            .color
            .interpolate(&start.detail.color, &end.detail.color, percent, elem, |s| {
                &s.detail.color
            });
        self.detail.texture.interpolate(
            &start.detail.texture,
            &end.detail.texture,
            percent,
            elem,
            |s| &s.detail.texture,
        );
        self.detail.shape.interpolate(
            &start.detail.shape,
            &end.detail.shape,
            percent,
            elem,
            |s| &s.detail.shape,
        );

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
        self.text
            .select_color
            .interpolate(&start.text.select_color, &end.text.select_color, percent, elem, |s| {
                &s.text.select_color
            });
        self.text.fit = (percent < 50f32).yn(start.text.fit.clone(), end.text.fit.clone());
        self.text.font = (percent < 50f32).yn(start.text.font.clone(), end.text.font.clone());

        //i wanted to write a funny comment but idk what to write
        // this is so overdone its funny - max
        //agreed.
        fn interpolate_align<E: UiElementStub, F: Fn(&UiStyle) -> &Resolve<TextAlign>>(elem: &E, a: Resolve<TextAlign>, b: Resolve<TextAlign>, i: f32, map: F) -> Resolve<TextAlign> {
            if i >= 100.0 {
                return b;
            } else if i <= 0.0 {
                return a;
            }
            let a = a.resolve(elem.state().ctx.dpi, elem.state().parent.clone(), |s| map(s));
            let b = b.resolve(elem.state().ctx.dpi, elem.state().parent.clone(), |s| map(s));
            let a = a.unwrap_or_default(map(&DEFAULT_STYLE));
            let b = b.unwrap_or_default(map(&DEFAULT_STYLE));
            let a: u8 = unsafe { mem::transmute(a) };
            let b: u8 = unsafe { mem::transmute(b) };
            let c = (a as f32 * i + b as f32 * (100.0 - i)) / 200.0;
            let align: TextAlign = unsafe { mem::transmute((c.round()) as u8) };
            UiValue::Just(align).to_resolve()
        }

        self.text.align_x = interpolate_align(elem, start.text.align_x.clone(), end.text.align_x.clone(), percent, |s| &s.text.align_x);
        self.text.align_y = interpolate_align(elem, start.text.align_y.clone(), end.text.align_y.clone(), percent, |s| &s.text.align_y);

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

        self.overflow_x = (percent > 50.0).yn(end.overflow_x.clone(), start.overflow_x.clone());

        self.overflow_y = (percent > 50.0).yn(end.overflow_y.clone(), start.overflow_y.clone());

        self.scrollbar.track.interpolate(
            &start.scrollbar.track,
            &end.scrollbar.track,
            percent,
            elem,
            |s| &f(s).scrollbar.track,
        );
        self.scrollbar.knob.interpolate(
            &start.scrollbar.knob,
            &end.scrollbar.knob,
            percent,
            elem,
            |s| &f(s).scrollbar.knob,
        );
        self.scrollbar.size.interpolate(
            &start.scrollbar.size,
            &end.scrollbar.size,
            percent,
            elem,
            |s| &f(s).scrollbar.size,
        );
    }
}

impl<Fmt: ColorFormat + 'static> Interpolator<Color<Fmt>> for Color<Fmt>
where
    Fmt::ComponentType: Interpolator<Fmt::ComponentType> + PartialOrd<Fmt::ComponentType>,
{
    fn interpolate<E, F>(&mut self, start: &Self, end: &Self, percent: f32, elem: &E, f: F)
    where
        E: UiElementStub,
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
