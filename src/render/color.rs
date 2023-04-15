use std::marker::PhantomData;
use regex::Regex;

pub trait Fmt {}
pub struct RGB {}
pub struct HSV {}

impl Fmt for RGB {}
impl Fmt for HSV {}

#[derive(Clone, Debug)]
pub struct Color<F: Fmt, T> {
    c1: T,
    c2: T,
    c3: T,
    c4: T,
    ignore: PhantomData<F>
}

pub trait Parse {
    fn parse(s: &str) -> Result<Self, &str> where Self: Sized;
}

impl Color<RGB, u8> {
    pub fn new() -> Self {
        Color {
            c1: 0,
            c2: 0,
            c3: 0,
            c4: 0,
            ignore: Default::default(),
        }
    }

    pub fn white() -> Self {Color::from4(255, 255, 255, 255)}
    pub fn black() -> Self {Color::from4(0, 0, 0, 0)}
    pub fn red() -> Self {Color::from4(255, 0, 0, 255)}
    pub fn green() -> Self {Color::from4(0, 255, 0, 255)}
    pub fn blue() -> Self {Color::from4(0, 0, 0, 255)}
    pub fn yellow() -> Self {Color::from4(255, 255, 0, 255)}
    pub fn magenta() -> Self {Color::from4(255, 0, 255, 255)}
    pub fn cyan() -> Self {Color::from4(0, 255, 255, 255)}

    pub fn from4(c1: u8, c2: u8, c3: u8, c4: u8) -> Self {
        Color {c1, c2, c3, c4, ignore: Default::default()}
    }

    pub fn r(&self) -> u8 {self.c1}
    pub fn g(&self) -> u8 {self.c2}
    pub fn b(&self) -> u8 {self.c3}
    pub fn a(&self) -> u8 {self.c4}

    pub fn set_r(&mut self, val: u8) {self.c1 = val}
    pub fn set_g(&mut self, val: u8) {self.c2 = val}
    pub fn set_b(&mut self, val: u8) {self.c3 = val}
    pub fn set_a(&mut self, val: u8) {self.c4 = val}

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
            ignore: Default::default()
        }
    }
}

impl Parse for Color<RGB, u8> {
    fn parse(s: &str) -> Result<Self, &str> {
        if s.starts_with("#") {
            let string = s.replace("#", "");
            if !Regex::new("-?[0-9a-fA-F]+").unwrap().is_match(s) {
                return Err("Color parser: # colors must be hexadecimal characters!");
            }
            let mut colors = string.split("(?<=\\G.{2})");
            if colors.clone().count() < 3 || colors.clone().count() > 4 {
                return Err("Color parser: # colors must contain 6 or 8 characters!");
            }
            let r = colors.next().unwrap().parse::<u8>().unwrap();
            let g = colors.next().unwrap().parse::<u8>().unwrap();
            let b = colors.next().unwrap().parse::<u8>().unwrap();
            let mut a: u8 = 255;
            if colors.clone().count() == 4 {
                let a = colors.next().unwrap().parse::<u8>().unwrap();
            }
            Ok(Color::from4(r, g, b, a))
        } else if s.starts_with("0x") {
            let string = s.replace("0x", "");
            if !Regex::new("-?[0-9a-fA-F]+").unwrap().is_match(s) {
                return Err("Color parser: # colors must be hexadecimal characters!");
            }
            let mut colors = string.split("(?<=\\G.{2})");
            if colors.clone().count() < 3 || colors.clone().count() > 4 {
                return Err("Color parser: # colors must contain 6 or 8 characters!");
            }
            let r = u8::from_str_radix(colors.next().unwrap(), 16).unwrap();
            let g = u8::from_str_radix(colors.next().unwrap(), 16).unwrap();
            let b = u8::from_str_radix(colors.next().unwrap(), 16).unwrap();
            let mut a: u8 = 255;
            if colors.clone().count() == 4 {
                let a = u8::from_str_radix(colors.next().unwrap(), 16);
            }
            Ok(Color::from4(r, g, b, a))
        } else {
            let mut split = ",";
            let mut string: String;
            string = s.to_string();
            if s.contains(" ") && s.contains(",") {
                string = s.replace(" ", "");
            } else if s.contains(" ") {
                split = " ";
            }

            let repl = string.replace(" ", ""); //I can't put this into one line, cuz this dumbass language doesn't allow it by freeing the temp var
            let mut colors = repl.split(split);
            if colors.clone().count() < 3 || colors.clone().count() > 4 {
                return Err("Color parser: colors must contain 3 or 4 sets of numbers!");
            }
            let r = colors.next().unwrap().parse::<u8>().unwrap();
            let g = colors.next().unwrap().parse::<u8>().unwrap();
            let b = colors.next().unwrap().parse::<u8>().unwrap();
            let mut a: u8 = 255;
            if colors.clone().count() == 4 {
                let a = colors.next().unwrap().parse::<u8>().unwrap();
            }
            Ok(Color::from4(r, g, b, a))
        }
    }
}