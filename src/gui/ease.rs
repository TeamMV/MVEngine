use std::ops::Range;
use mvutils::utils::Map;

pub struct Easing {
    pub start: f32, //        __t
    pub end: f32,   //    ___/
    pub from: f32,  //f__/
    pub to: f32,    //s         e
    gen: EasingGen
}

impl Easing {
    pub fn get(&self, x: f32) -> f32 {
        self.gen.get(x)
    }

    pub fn simulate(&self, range: &Range<f32>, steps: usize) -> &[f32] {
        self.gen.simulate(range, steps)
    }
}

pub enum EasingGen {
    Linear(EasingLinear)
}

macro_rules! ease_fn {
    ($s:expr, $name:ident, $($param:ident),*) => {
        return match $s {
            EasingGen::Linear(e) => {e.$name($($param,)*)}
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
        let step: f32 = (range.end - range.start) / steps;
        let mut i: f32 = 0.0;
        while i < range.end {
            vec.push(self.get(i));
            i += step;
        }
        vec.as_slice()
    }

    fn xr(&self) -> Range<f32>;
    fn yr(&self) -> Range<f32>;
}

pub struct EasingLinear {
    xr: Range<f32>,
    yr: Range<f32>,
}

impl EasingFunction for EasingLinear {
    fn get(&self, x: f32) -> f32 {
        x.map(self.yr(), self.xr())
    }

    fn xr(&self) -> &Range<f32> {
        &self.xr
    }

    fn yr(&self) -> &Range<f32> {
        &self.yr
    }
}