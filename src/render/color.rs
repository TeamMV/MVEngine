use std::marker::PhantomData;
use mvutils::utils::SplitSized;

use regex::Regex;

pub trait Fmt {}
#[derive(Debug, Default, Eq, PartialEq, Copy, Clone)]
pub struct RGB;
#[derive(Debug, Default, Eq, PartialEq, Copy, Clone)]
pub struct HSV;
impl Fmt for RGB {}
impl Fmt for HSV {}

#[derive(Clone, Debug)]
pub struct Color<F: Fmt, T> {
    c1: T,
    c2: T,
    c3: T,
    c4: T,
    phantom: PhantomData<F>,
}

impl<U: Fmt, T: Default> Default for Color<U, T> {
    fn default() -> Self {
        Color {
            c1: T::default(),
            c2: T::default(),
            c3: T::default(),
            c4: T::default(),
            phantom: Default::default(),
        }
    }
}

impl<U: Fmt, T: Clone> Color<U, T> {
    pub fn copy_of(&mut self, other: &Color<U, T>) {
        self.c1 = other.c1.clone();
        self.c2 = other.c2.clone();
        self.c3 = other.c3.clone();
        self.c4 = other.c4.clone();
    }
}

impl<U: Fmt, T> Color<U, T> {
    pub fn new(c1: T, c2: T, c3: T, c4: T) -> Self {
        Color { c1, c2, c3, c4, phantom: Default::default() }
    }
}

impl<U: Fmt, T: Clone> Clone for Color<U, T> {
    fn clone(&self) -> Self {
        Color {
            c1: self.c1.clone(),
            c2: self.c2.clone(),
            c3: self.c3.clone(),
            c4: self.c4.clone(),
            phantom: Default::default(),
        }
    }
}

impl<U: Fmt, T: Copy> Copy for Color<U, T> {}

impl<T: Copy> Color<RGB, T> {
    pub fn r(&self) -> T { self.c1 }
    pub fn g(&self) -> T { self.c2 }
    pub fn b(&self) -> T { self.c3 }
    pub fn a(&self) -> T { self.c4 }

    pub fn set_r(&mut self, val: T) { self.c1 = val }
    pub fn set_g(&mut self, val: T) { self.c2 = val }
    pub fn set_b(&mut self, val: T) { self.c3 = val }
    pub fn set_a(&mut self, val: T) { self.c4 = val }

    pub fn set(&mut self, r: T, g: T, b: T, a: T) {
        self.c1 = r;
        self.c2 = g;
        self.c3 = b;
        self.c4 = a;
    }
}

pub trait Parse {
    fn parse(s: &str) -> Result<Self, &str> where Self: Sized;
}

impl Color<RGB, u8> {
    pub fn white() -> Self { Color::new(255, 255, 255, 255) }
    pub fn black() -> Self { Color::new(0, 0, 0, 0) }
    pub fn red() -> Self { Color::new(255, 0, 0, 255) }
    pub fn green() -> Self { Color::new(0, 255, 0, 255) }
    pub fn blue() -> Self { Color::new(0, 0, 0, 255) }
    pub fn yellow() -> Self { Color::new(255, 255, 0, 255) }
    pub fn magenta() -> Self { Color::new(255, 0, 255, 255) }
    pub fn cyan() -> Self { Color::new(0, 255, 255, 255) }

    pub fn normalize(self) -> Color<RGB, f32> {
        Color {
            c1: self.r() as f32 / 255.0,
            c2: self.g() as f32 / 255.0,
            c3: self.b() as f32 / 255.0,
            c4: self.a() as f32 / 255.0,
            phantom: Default::default(),
        }
    }

    fn pv(colors: Vec<String>, radix: u32) -> Result<Self, ()> {
        if colors.len() < 3 || colors.len() > 4 {
            return Err(());
        }
        let r = u8::from_str_radix(&colors[0], radix).map_err(|_| ())?;
        let g = u8::from_str_radix(&colors[1], radix).map_err(|_| ())?;
        let b = u8::from_str_radix(&colors[2], radix).map_err(|_| ())?;
        let mut a: u8 = 255;
        if colors.len() == 4 {
            a = u8::from_str_radix(&colors[3], radix).map_err(|_| ())?;
        }
        Ok(Color::new(r, g, b, a))
    }

    pub fn copy_hsv(&mut self, col: &Color<HSV, f32>) {
        let c = col.c2 * col.c3;
        let h = col.c1 / 60.0;
        let x = c * (1.0 - (h % 2.0 - 1.0).abs());
        let m = col.c3 - col.c2;

        let rgb = match h.floor() as u8 {
            0 => [c, x, 0.0],
            1 => [x, c, 0.0],
            2 => [0.0, c, x],
            3 => [0.0, x, c],
            4 => [x, 0.0, c],
            5 | 6 => [c, 0.0, x],
            _ => panic!("Illegal color!")
        };
        let r = ((rgb[0] + m) * 255.0) as u8;
        let g = ((rgb[1] + m) * 255.0) as u8;
        let b = ((rgb[2] + m) * 255.0) as u8;
        let a = (col.c4 * 255.0) as u8;
        self.set(r, g, b, a);
    }

    pub fn from_hsv(col: Color<HSV, f32>) -> Self {
        let mut c: Color<RGB, u8> = Color::default();
        c.copy_hsv(&col);
        c
    }
}

impl Color<RGB, f32> {
    pub fn white() -> Self { Color::new(1.0, 1.0, 1.0, 1.0) }
    pub fn black() -> Self { Color::new(0.0, 0.0, 0.0, 0.0) }
    pub fn red() -> Self { Color::new(1.0, 0.0, 0.0, 1.0) }
    pub fn green() -> Self { Color::new(0.0, 1.0, 0.0, 1.0) }
    pub fn blue() -> Self { Color::new(0.0, 0.0, 0.0, 1.0) }
    pub fn yellow() -> Self { Color::new(1.0, 1.0, 0.0, 1.0) }
    pub fn magenta() -> Self { Color::new(1.0, 0.0, 1.0, 1.0) }
    pub fn cyan() -> Self { Color::new(0.0, 1.0, 1.0, 1.0) }

    pub fn normalize(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.c1 = r as f32 / 255.0;
        self.c2 = g as f32 / 255.0;
        self.c3 = b as f32 / 255.0;
        self.c4 = a as f32 / 255.0;
    }

    pub fn denormalize(self) -> Color<RGB, u8> {
        Color {
            c1: (self.r() * 255.0) as u8,
            c2: (self.g() * 255.0) as u8,
            c3: (self.b() * 255.0) as u8,
            c4: (self.a() * 255.0) as u8,
            phantom: Default::default(),
        }
    }

    pub fn copy_hsv(&mut self, col: &Color<HSV, f32>) {
        let c = col.c2 * col.c3;
        let h = col.c1 / 60.0;
        let x = c * (1.0 - (h % 2.0 - 1.0).abs());
        let m = col.c3 - col.c2;

        let rgb = match h.floor() as u8 {
            0 => [c, x, 0.0],
            1 => [x, c, 0.0],
            2 => [0.0, c, x],
            3 => [0.0, x, c],
            4 => [x, 0.0, c],
            5 | 6 => [c, 0.0, x],
            _ => panic!("Illegal color!")
        };
        self.set(rgb[0] + m, rgb[1] + m, rgb[2] + m, col.c4)
    }

    pub fn from_hsv(col: Color<HSV, f32>) -> Self {
        let mut c: Color<RGB, f32> = Color::default();
        c.copy_hsv(&col);
        c
    }
}

impl<T: Copy> Color<HSV, T> {
    pub fn h(&self) -> T { self.c1 }
    pub fn s(&self) -> T { self.c2 }
    pub fn v(&self) -> T { self.c3 }
    pub fn a(&self) -> T { self.c4 }

    pub fn set_h(&mut self, val: T) { self.c1 = val }
    pub fn set_s(&mut self, val: T) { self.c2 = val }
    pub fn set_v(&mut self, val: T) { self.c3 = val }
    pub fn set_a(&mut self, val: T) { self.c4 = val }

    pub fn set(&mut self, h: T, s: T, v: T, a: T) {
        self.c1 = h;
        self.c2 = s;
        self.c3 = v;
        self.c4 = a;
    }
}

impl Color<HSV, f32> {
    pub fn as_rgb(&self) -> Color<RGB, f32> {
        let c = self.c2 * self.c3;
        let h = self.c1 / 60.0;
        let x = c * (1.0 - (h % 2.0 - 1.0).abs());
        let m = self.c3 - self.c2;

        let rgb = match h.floor() as u8 {
            0 => [c, x, 0.0],
            1 => [x, c, 0.0],
            2 => [0.0, c, x],
            3 => [0.0, x, c],
            4 => [x, 0.0, c],
            5 | 6 => [c, 0.0, x],
            _ => panic!("Illegal color!")
        };
        Color::new(rgb[0] + m, rgb[1] + m, rgb[2] + m, self.c4)
    }
}

impl Parse for Color<RGB, u8> {
    fn parse(s: &str) -> Result<Self, &str> {
        if s.starts_with('#') || s.starts_with("0x") {
            let s = s.replace('#', "").replace("0x", "");
            if !Regex::new("^([0-9a-fA-F]{2}){3,4}$").unwrap().is_match(&s) {
                return Err("Color xml: # and 0x colors must be hexadecimal characters!");
            }
            let colors = s.split_sized(2);
            Self::pv(colors, 16).map_err(|_| "Color xml: # colors must contain 6 or 8 characters!")
        } else {
            let colors = if s.contains(',') {
                let string = s.replace(' ', "");
                string.split(',').map(str::to_string).collect::<Vec<_>>()
            } else if s.contains(' ') {
                s.split_whitespace().map(str::to_string).collect::<Vec<_>>()
            } else {
                return Err("Color xml: color values must be separated by space or comma!");
            };
            Self::pv(colors, 10).map_err(|_| "Color must have 3 or 4 parts!")
        }
    }
}