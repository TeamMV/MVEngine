use std::marker::PhantomData;

use regex::Regex;

pub trait Fmt {}

pub struct RGB {}

pub struct HSV {}

impl Fmt for RGB {}

impl Fmt for HSV {}

#[derive(Clone, Debug)]
pub struct Color<F: Fmt, T: Default + Clone> {
    c1: T,
    c2: T,
    c3: T,
    c4: T,
    ignore: PhantomData<F>,
}

impl<U: Fmt, T: Default + Clone> Color<U, T> {
    pub fn new() -> Self {
        Color {
            c1: T::default(),
            c2: T::default(),
            c3: T::default(),
            c4: T::default(),
            ignore: Default::default(),
        }
    }

    pub fn copy_of(&mut self, other: &Color<U, T>) {
        self.c1 = other.c1.clone();
        self.c2 = other.c2.clone();
        self.c3 = other.c3.clone();
        self.c4 = other.c4.clone();
    }
}

pub trait Parse {
    fn parse(s: &str) -> Result<Self, &str> where Self: Sized;
}

impl Color<RGB, u8> {
    pub fn white() -> Self { Color::from4(255, 255, 255, 255) }
    pub fn black() -> Self { Color::from4(0, 0, 0, 0) }
    pub fn red() -> Self { Color::from4(255, 0, 0, 255) }
    pub fn green() -> Self { Color::from4(0, 255, 0, 255) }
    pub fn blue() -> Self { Color::from4(0, 0, 0, 255) }
    pub fn yellow() -> Self { Color::from4(255, 255, 0, 255) }
    pub fn magenta() -> Self { Color::from4(255, 0, 255, 255) }
    pub fn cyan() -> Self { Color::from4(0, 255, 255, 255) }

    pub fn from4(c1: u8, c2: u8, c3: u8, c4: u8) -> Self {
        Color { c1, c2, c3, c4, ignore: Default::default() }
    }

    pub fn r(&self) -> u8 { self.c1 }
    pub fn g(&self) -> u8 { self.c2 }
    pub fn b(&self) -> u8 { self.c3 }
    pub fn a(&self) -> u8 { self.c4 }

    pub fn set_r(&mut self, val: u8) { self.c1 = val }
    pub fn set_g(&mut self, val: u8) { self.c2 = val }
    pub fn set_b(&mut self, val: u8) { self.c3 = val }
    pub fn set_a(&mut self, val: u8) { self.c4 = val }

    pub fn set(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.c1 = r;
        self.c2 = g;
        self.c3 = b;
        self.c4 = a;
    }

    pub fn normalize(self) -> Color<RGB, f32> {
        Color {
            c1: self.r() as f32 / 255.0,
            c2: self.g() as f32 / 255.0,
            c3: self.b() as f32 / 255.0,
            c4: self.a() as f32 / 255.0,
            ignore: Default::default(),
        }
    }

    fn pv(colors: Vec<String>) -> Result<Self, ()> {
        if colors.len() < 3 || colors.len() > 4 {
            return Err(());
        }
        let r = colors[0].parse::<u8>().unwrap();
        let g = colors[1].parse::<u8>().unwrap();
        let b = colors[2].parse::<u8>().unwrap();
        let mut a: u8 = 255;
        if colors.len() == 4 {
            a = colors[3].parse::<u8>().unwrap();
        }
        Ok(Color::from4(r, g, b, a))
    }
}

impl Color<HSV, u16> {
    pub fn h(&self) -> u16 { self.c1 }
    pub fn s(&self) -> u16 { self.c2 }
    pub fn v(&self) -> u16 { self.c3 }
    pub fn a(&self) -> u16 { self.c4 }

    pub fn set_h(&mut self, val: u16) { self.c1 = val }
    pub fn set_s(&mut self, val: u16) { self.c2 = val }
    pub fn set_v(&mut self, val: u16) { self.c3 = val }
    pub fn set_a(&mut self, val: u16) { self.c4 = val }

    pub fn set(&mut self, h: u16, s: u16, v: u16, a: u16) {
        self.c1 = h;
        self.c2 = s;
        self.c3 = v;
        self.c4 = a;
    }
}

impl Parse for Color<RGB, u8> {
    fn parse(s: &str) -> Result<Self, &str> {
        if s.starts_with("#") || s.starts_with("0x") {
            let s = s.replace("#", "").replace("0x", "");
            if !Regex::new("([0-9a-fA-F]{2}){3,4}").unwrap().is_match(&*s) {
                return Err("Color xml: # and 0x colors must be hexadecimal characters!");
            }
            let colors = Regex::new("(?<=\\G.{2})").unwrap().split(&*s).map(str::to_string).collect::<Vec<_>>();
            Self::pv(colors).map_err(|e| "Color xml: # colors must contain 6 or 8 characters!")
        } else {
            let colors = if s.contains(",") {
                let string = s.replace(" ", "");
                string.split(",").map(str::to_string).collect::<Vec<_>>()
            } else if s.contains(" ") {
                s.split_whitespace().map(str::to_string).collect::<Vec<_>>()
            } else {
                return Err("Color xml: color values must be separated by space or comma!");
            };
            Self::pv(colors).map_err(|e| "Color must have 3 or 4 parts!")
        }
    }
}