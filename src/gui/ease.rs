use std::f32::consts::PI;
use std::ops::Range;
use mvutils::utils::{Map, Overlap};

#[derive(Clone)]
pub struct Easing {
    pub xr: Range<f32>,
    pub yr: Range<f32>,
    gen: EasingGen
}

impl Easing {
    pub fn new_linear(xr: Range<f32>, yr: Range<f32>) -> Self {
        Self {
            xr,
            yr,
            gen: EasingGen::Linear(EasingLinear::new(xr.clone(), yr.clone())),
        }
    }

    pub fn new_sin(xr: Range<f32>, yr: Range<f32>) -> Self {
        Self {
            xr,
            yr,
            gen: EasingGen::Sin(EasingSin::new(xr.clone(), yr.clone())),
        }
    }

    pub fn new_sin_in(xr: Range<f32>, yr: Range<f32>) -> Self {
        Self {
            xr,
            yr,
            gen: EasingGen::SinIn(EasingSinIn::new(xr.clone(), yr.clone())),
        }
    }

    pub fn new_sin_out(xr: Range<f32>, yr: Range<f32>) -> Self {
        Self {
            xr,
            yr,
            gen: EasingGen::SinOut(EasingSinOut::new(xr.clone(), yr.clone())),
        }
    }

    pub fn get(&self, x: f32) -> f32 {
        self.gen.get(x)
    }

    pub fn simulate(&self, range: &Range<f32>, steps: usize) -> &[f32] {
        self.gen.simulate(range, steps)
    }
}

impl Default for Easing {
    fn default() -> Self {
        Easing::new_linear(0.0..1.0, 0.0..1.0)
    }
}

#[derive(Clone)]
pub enum EasingGen {
    Linear(EasingLinear),
    Sin(EasingSin),
    SinIn(EasingSinIn),
    SinOut(EasingSinOut),
    ExplIn(EasingExpIn)
}

macro_rules! ease_fn {
    ($s:expr, $name:ident, $($param:ident),*) => {
        return match $s {
            EasingGen::Linear(e) => {e.$name($($param,)*)}
            EasingGen::Sin(e) => {e.$name($($param,)*)}
            EasingGen::SinIn(e) => {e.$name($($param,)*)}
            EasingGen::SinOut(e) => {e.$name($($param,)*)}
            EasingGen::ExpIn(e) => {e.$name($($param,)*)}
            _ => {unreachable!()}
        }
    };
}

impl EasingGen {
    pub fn get(&self, x: f32) -> f32 {
        ease_fn!(self, get, x)
    }

    pub fn simulate(&self, range: &Range<f32>, steps: usize) -> &[f32] {
        ease_fn!(self, simulate, range, steps)
    }
}

pub trait EasingFunction {
    fn get(&self, x: f32) -> f32;

    fn simulate(&self, range: &Range<f32>, steps: usize) -> &[f32] {
        let mut vec: Vec<f32> = vec![];
        let step: f32 = (range.end - range.start) / steps as f32;
        let mut i: f32 = 0.0;
        while i < range.end {
            vec.push(self.get(i));
            i += step;
        }
        vec.as_slice()
    }
}

macro_rules! easing_struct {
    ($name:ident $(,$p:ident: $t:ty)*) => {
        #[derive(Clone)]
        pub struct $name {
            pub(crate) xr: Range<f32>,
            pub(crate) yr: Range<f32>,
            $(
                pub(crate) $p: $t
            ),*
        }

        impl $name {
            pub fn new(xr: Range<f32>, yr: Range<f32> $(, $p: $t)*) -> Self {
                Self {
                    xr,
                    yr,
                    $(
                        $p
                    ),*
                }
            }
        }
    };
}

easing_struct!(EasingLinear);
easing_struct!(EasingSin);
easing_struct!(EasingSinIn);
easing_struct!(EasingSinOut);
easing_struct!(EasingExpIn, base: f32);

impl EasingFunction for EasingLinear {
    fn get(&self, x: f32) -> f32 {
        if x <= self.xr.start {
            return self.yr.start;
        }
        else if x >= self.xr.end {
            return self.yr.end;
        }
        x.map(&self.yr, &self.xr)
    }
}

impl EasingFunction for EasingSin {
    fn get(&self, x: f32) -> f32 {
        if x <= self.xr.start {
            return self.yr.start;
        }
        else if x >= self.xr.end {
            return self.yr.end;
        }
        ((f32::cos((PI * (x - self.yr.start)) / (self.yr.end - self.yr.start) + PI) + 1.0) * ((self.xr.end - self.xr.start) / 2.0) + self.xr.start)
    }
}

impl EasingFunction for EasingSinIn {
    fn get(&self, x: f32) -> f32 {
        if x <= self.xr.start {
            return self.yr.start;
        }
        else if x >= self.xr.end {
            return self.yr.end;
        }
        ((f32::cos((PI * (x - self.yr.start)) / (self.yr.end - self.yr.start) + PI) + 1.0) * (self.xr.end - self.xr.start) + self.xr.start)
    }
}

impl EasingFunction for EasingSinOut {
    fn get(&self, x: f32) -> f32 {
        if x <= self.xr.start {
            return self.yr.start;
        }
        else if x >= self.xr.end {
            return self.yr.end;
        }
        ((f32::cos((PI * (x - self.yr.start)) / (2.0 * (self.yr.end - self.yr.start)) + PI) + 1.0) * (self.xr.end - self.xr.start) + self.xr.start)
    }
}

impl EasingFunction for EasingExpIn {
    fn get(&self, x: f32) -> f32 {
        if x <= self.xr.start {
            return self.yr.start;
        }
        else if x >= self.xr.end {
            return self.yr.end;
        }
        (self.yr.end - self.yr.start) * ((self.base.powf(x / (self.xr.end - self.xr.start) - self.xr.start) - 1) / (x - 1)) + self.yr.start
    }
}

pub fn linear(x: f32, xr: &Range<f32>, yr: &Range<f32>) -> f32 {
    if x <= xr.start {
        return yr.start;
    }
    else if x >= xr.end {
        return yr.end;
    }
    x.map(yr, xr)
}

pub fn sin(x: f32, xr: &Range<f32>, yr: &Range<f32>) -> f32 {
    if x <= xr.start {
        return yr.start;
    }
    else if x >= xr.end {
        return yr.end;
    }
    ((f32::cos((PI * (x - yr.start)) / (yr.end - yr.start) + PI) + 1.0) * ((xr.end - xr.start) / 2.0) + xr.start)
}

pub fn sin_in(x: f32, xr: &Range<f32>, yr: &Range<f32>) -> f32 {
    if x <= xr.start {
        return yr.start;
    }
    else if x >= xr.end {
        return yr.end;
    }
    ((f32::cos((PI * (x - yr.start)) / (yr.end - yr.start) + PI) + 1.0) * (xr.end - xr.start) + xr.start)
}

pub fn sin_out(x: f32, xr: &Range<f32>, yr: &Range<f32>) -> f32 {
    if x <= xr.start {
        return yr.start;
    }
    else if x >= xr.end {
        return yr.end;
    }
    ((f32::cos((PI * (x - yr.start)) / (2.0 * (yr.end - yr.start)) + PI) + 1.0) * (xr.end - xr.start) + xr.start)
}

pub fn exp_in(x: f32, xr: &Range<f32>, yr: &Range<f32>, base: f32) -> f32 {
    if x <= xr.start {
        return yr.start;
    }
    else if x >= xr.end {
        return yr.end;
    }
    (yr.end - yr.start) * ((base.powf(x / (xr.end - xr.start) - xr.start) - 1) / (x - 1)) + yr.start
}
