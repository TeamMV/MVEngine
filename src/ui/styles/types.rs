use num_traits::{Num, One};
use std::fmt::Debug;
use std::cmp::Ordering;
use std::ops::Add;
use crate::ui::elements::UiElementStub;
use crate::ui::styles::UiStyle;
use crate::ui::styles::interpolate::Interpolator;

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

impl<T: Interpolator<T> + Num + Clone + Debug + PartialOrd + 'static> Interpolator<Dimension<T>>
    for Dimension<T>
{
    fn interpolate<E, F>(&mut self, start: &Self, end: &Dimension<T>, percent: f32, elem: &E, f: F)
    where
        E: UiElementStub,
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
    fn interpolate<E, F>(&mut self, start: &Self, end: &Point<T>, percent: f32, elem: &E, f: F)
    where
        E: UiElementStub,
        F: Fn(&UiStyle) -> &Self,
    {
        self.x
            .interpolate(&start.x, &end.x, percent, elem, |s| &f(s).x);
        self.y
            .interpolate(&start.y, &end.y, percent, elem, |s| &f(s).y);
    }
}