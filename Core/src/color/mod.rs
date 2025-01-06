pub mod utils;
pub mod parse;

use crate::math::vec::Vec4;
use num_traits::{Num, ToPrimitive};
use std::cmp::Ordering;
use std::ops::Div;

pub type RgbColor = Color<RgbColorFormat>;
pub type HsvColor = Color<HsvColorFormat>;

pub trait ColorFormat
where
    Self: Sized,
{
    type ComponentType: Copy;

    fn get_rgb(color: Color<Self>) -> RgbColor;

    fn from_rgb(color: RgbColor) -> Color<Self>;
}

#[derive(Debug)]
pub struct Color<Fmt: ColorFormat> {
    components: [Fmt::ComponentType; 4],
}

impl<T: ColorFormat> Clone for Color<T> {
    fn clone(&self) -> Self {
        Self {
            components: self.components.clone(),
        }
    }
}

impl<Fmt: ColorFormat> Color<Fmt> {
    pub fn new(components: [Fmt::ComponentType; 4]) -> Self {
        Self { components }
    }

    pub fn to_rgb(self) -> RgbColor {
        Fmt::get_rgb(self)
    }

    pub fn to_hsv(self) -> HsvColor {
        HsvColorFormat::from_rgb(Fmt::get_rgb(self))
    }

    pub fn components(&self) -> &[Fmt::ComponentType] {
        &self.components
    }

    pub fn components_mut(&mut self) -> &mut [Fmt::ComponentType] {
        &mut self.components
    }

    pub fn alpha(mut self, alpha: Fmt::ComponentType) -> Self {
        self.components[3] = alpha;
        self
    }
}

impl<Fmt: ColorFormat> PartialEq<Self> for Color<Fmt>
where
    Fmt::ComponentType: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.components.iter().eq(other.components.iter())
    }
}

impl<Fmt: ColorFormat> PartialOrd for Color<Fmt>
where
    Fmt::ComponentType: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.components.iter().partial_cmp(other.components.iter())
    }
}

impl<Fmt: ColorFormat> Color<Fmt>
where
    Fmt::ComponentType: ToPrimitive,
{
    pub fn as_vec4(&self) -> Vec4 {
        Vec4::new(
            self.components[0].to_f32().unwrap() / 255.0,
            self.components[1].to_f32().unwrap() / 255.0,
            self.components[2].to_f32().unwrap() / 255.0,
            self.components[3].to_f32().unwrap() / 255.0,
        )
    }
}

impl RgbColor {
    pub fn white() -> Self {
        Color::new([255, 255, 255, 255])
    }
    pub fn black() -> Self {
        Color::new([0, 0, 0, 255])
    }
    pub fn red() -> Self {
        Color::new([255, 0, 0, 255])
    }
    pub fn green() -> Self {
        Color::new([0, 255, 0, 255])
    }
    pub fn blue() -> Self {
        Color::new([0, 0, 255, 255])
    }
    pub fn yellow() -> Self {
        Color::new([255, 255, 0, 255])
    }
    pub fn magenta() -> Self {
        Color::new([255, 0, 255, 255])
    }
    pub fn cyan() -> Self {
        Color::new([0, 255, 255, 255])
    }
    pub fn transparent() -> Self {
        Color::new([0, 0, 0, 0])
    }
}

#[derive(Debug)]
pub struct RgbColorFormat {}

impl ColorFormat for RgbColorFormat {
    type ComponentType = u8;

    fn get_rgb(color: Color<Self>) -> Color<RgbColorFormat> {
        color
    }

    fn from_rgb(color: RgbColor) -> Color<Self> {
        color
    }
}

pub struct HsvColorFormat {}

impl ColorFormat for HsvColorFormat {
    type ComponentType = f32;

    fn get_rgb(color: Color<Self>) -> RgbColor {
        let [h, s, v, a] = color.components;

        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r1, g1, b1) = match h {
            0.0..=60.0 => (c, x, 0.0),
            60.0..=120.0 => (x, c, 0.0),
            120.0..=180.0 => (0.0, c, x),
            180.0..=240.0 => (0.0, x, c),
            240.0..=300.0 => (x, 0.0, c),
            300.0..=360.0 => (c, 0.0, x),
            _ => (0.0, 0.0, 0.0),
        };

        let r = ((r1 + m) * 255.0).round() as u8;
        let g = ((g1 + m) * 255.0).round() as u8;
        let b = ((b1 + m) * 255.0).round() as u8;

        RgbColor::new([r, g, b, (a * 255.0) as u8])
    }

    fn from_rgb(color: RgbColor) -> Color<Self> {
        let [r, g, b, a] = color.components.map(|c| c as f32 / 255.0);

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        let h = if h < 0.0 { h + 360.0 } else { h };

        let s = if max == 0.0 { 0.0 } else { delta / max };
        let v = max;

        Color::new([h, s, v, a])
    }
}
