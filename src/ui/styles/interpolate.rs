use crate::color::{Color, ColorFormat};
use crate::ui::elements::UiElementStub;
use crate::ui::styles::{Resolve, UiStyle, UiStyleInner, UiValue, DEFAULT_STYLE};
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

impl Interpolator<UiStyleInner> for UiStyleInner {
    fn interpolate<E, F>(&mut self, start: &Self, end: &Self, percent: f32, elem: &E, f: F)
    where
        E: UiElementStub,
        F: Fn(&UiStyle) -> &Self,
    {
        self.x
            .interpolate(&start.x, &end.x, percent, elem, |s| &f(s).x);
        self.y
            .interpolate(&start.y, &end.y, percent, elem, |s| &f(s).y);
        self.width
            .interpolate(&start.width, &end.width, percent, elem, |s| &f(s).width);
        self.height
            .interpolate(&start.height, &end.height, percent, elem, |s| &f(s).height);

        self.padding
            .interpolate(&start.padding, &end.padding, percent, elem, |s| &f(s).padding);
        self.margin
            .interpolate(&start.margin, &end.margin, percent, elem, |s| &f(s).margin);

        self.origin = (percent < 50f32).yn(start.origin.clone(), end.origin.clone());
        self.position = (percent < 50f32).yn(start.position.clone(), end.position.clone());
        self.direction = (percent < 50f32).yn(start.direction.clone(), end.direction.clone());
        self.child_align_x =
            (percent < 50f32).yn(start.child_align_x.clone(), end.child_align_x.clone());
        self.child_align_y =
            (percent < 50f32).yn(start.child_align_y.clone(), end.child_align_y.clone());

        self.background.interpolate(
            &start.background,
            &start.background,
            percent,
            elem,
            |s| &f(s).background,
        );
        self.border
            .interpolate(&start.border, &start.border, percent, elem, |s| &f(s).border);
        self.detail
            .interpolate(&start.detail, &start.detail, percent, elem, |s| &f(s).detail);

        self.text
            .interpolate(&start.text, &end.text, percent, elem, |s| &f(s).text);

        //i wanted to write a funny comment but idk what to write
        // this is so overdone its funny - max
        //agreed.
        //thats why i remade it now finally 11.09.2025 23:05

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
