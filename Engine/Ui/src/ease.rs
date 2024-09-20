use mvcore::math::curve::SimpleBezierCurve;
use mvutils::utils::Map;
use std::ops::Range;

#[derive(Clone)]
pub struct Easing {
    gen: EasingGen,
    mode: EasingMode,
    x_range: Range<f32>,
    y_range: Range<f32>,
}

impl Easing {
    pub fn new(gen: EasingGen, mode: EasingMode, x_range: Range<f32>, y_range: Range<f32>) -> Self {
        Self {
            gen,
            mode,
            x_range,
            y_range,
        }
    }

    pub fn get(&self, pos: f32) -> f32 {
        match &self.gen {
            EasingGen::Linear(e) => e.get(
                pos,
                self.x_range.clone(),
                self.y_range.clone(),
                self.mode.clone(),
            ),
            EasingGen::Exponential(e) => e.get(
                pos,
                self.x_range.clone(),
                self.y_range.clone(),
                self.mode.clone(),
            ),
            EasingGen::Sin(e) => e.get(
                pos,
                self.x_range.clone(),
                self.y_range.clone(),
                self.mode.clone(),
            ),
            EasingGen::Back(e) => e.get(
                pos,
                self.x_range.clone(),
                self.y_range.clone(),
                self.mode.clone(),
            ),
            EasingGen::Bounce(e) => e.get(
                pos,
                self.x_range.clone(),
                self.y_range.clone(),
                self.mode.clone(),
            ),
            EasingGen::Elastic(e) => e.get(
                pos,
                self.x_range.clone(),
                self.y_range.clone(),
                self.mode.clone(),
            ),
            EasingGen::Bezier(e) => e.get(pos.map(&self.x_range, &(0f32..1f32)) as f64) as f32,
        }
    }
}

#[derive(Clone)]
pub enum EasingGen {
    Linear(LinearEasing),
    Exponential(ExponentialEasing),
    Sin(SinEasing),
    Back(BackEasing),
    Bounce(BounceEasing),
    Elastic(ElasticEasing),
    Bezier(SimpleBezierCurve),
}

impl EasingGen {
    pub fn linear() -> Self {
        Self::Linear(LinearEasing)
    }
    pub fn exponential(exp: f32) -> Self {
        Self::Exponential(ExponentialEasing { exponent: exp })
    }
    pub fn sin() -> Self {
        Self::Sin(SinEasing)
    }
    pub fn back() -> Self {
        Self::Back(BackEasing)
    }
    pub fn bounce() -> Self {
        Self::Bounce(BounceEasing)
    }
    pub fn elastic() -> Self {
        Self::Elastic(ElasticEasing)
    }
    pub fn bezier(points: &[f64]) -> Self {
        Self::Bezier(SimpleBezierCurve::new(points))
    }
}

#[derive(Clone)]
pub enum EasingMode {
    In,
    Out,
    InOut,
}

pub trait EasingFunction {
    fn get(&self, pos: f32, x_range: Range<f32>, y_range: Range<f32>, mode: EasingMode) -> f32;
}

// Linear Easing
#[derive(Clone)]
pub struct LinearEasing;

impl EasingFunction for LinearEasing {
    fn get(&self, pos: f32, x_range: Range<f32>, y_range: Range<f32>, mode: EasingMode) -> f32 {
        let t = (pos - x_range.start) / (x_range.end - x_range.start);
        let value = t;
        y_range.start + value * (y_range.end - y_range.start)
    }
}

// Exponential Easing
#[derive(Clone)]
pub struct ExponentialEasing {
    pub exponent: f32,
}

impl EasingFunction for ExponentialEasing {
    fn get(&self, pos: f32, x_range: Range<f32>, y_range: Range<f32>, mode: EasingMode) -> f32 {
        let t = (pos - x_range.start) / (x_range.end - x_range.start);
        let value = match mode {
            EasingMode::In => {
                if t == 0.0 {
                    0.0
                } else {
                    (self.exponent).powf(10.0 * (t - 1.0))
                }
            }
            EasingMode::Out => {
                if t == 1.0 {
                    1.0
                } else {
                    1.0 - (self.exponent).powf(-10.0 * t)
                }
            }
            EasingMode::InOut => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    0.5 * (self.exponent).powf(20.0 * t - 10.0)
                } else {
                    1.0 - 0.5 * (self.exponent).powf(-20.0 * t + 10.0)
                }
            }
        };
        y_range.start + value * (y_range.end - y_range.start)
    }
}

// Sinusoidal Easing
#[derive(Clone)]
pub struct SinEasing;

impl EasingFunction for SinEasing {
    fn get(&self, pos: f32, x_range: Range<f32>, y_range: Range<f32>, mode: EasingMode) -> f32 {
        let t = (pos - x_range.start) / (x_range.end - x_range.start);
        let value = match mode {
            EasingMode::In => 1.0 - (t * std::f32::consts::FRAC_PI_2).cos(),
            EasingMode::Out => (t * std::f32::consts::FRAC_PI_2).sin(),
            EasingMode::InOut => 0.5 * (1.0 - (std::f32::consts::PI * t).cos()),
        };
        y_range.start + value * (y_range.end - y_range.start)
    }
}

// Back Easing
#[derive(Clone)]
pub struct BackEasing;

impl EasingFunction for BackEasing {
    fn get(&self, pos: f32, x_range: Range<f32>, y_range: Range<f32>, mode: EasingMode) -> f32 {
        let t = (pos - x_range.start) / (x_range.end - x_range.start);
        let s = 1.70158;
        let value = match mode {
            EasingMode::In => t * t * ((s + 1.0) * t - s),
            EasingMode::Out => {
                let t = t - 1.0;
                t * t * ((s + 1.0) * t + s) + 1.0
            }
            EasingMode::InOut => {
                let s = s * 1.525;
                if t < 0.5 {
                    let t = 2.0 * t;
                    0.5 * (t * t * ((s + 1.0) * t - s))
                } else {
                    let t = 2.0 * t - 2.0;
                    0.5 * (t * t * ((s + 1.0) * t + s) + 2.0)
                }
            }
        };
        y_range.start + value * (y_range.end - y_range.start)
    }
}

// Bounce Easing
#[derive(Clone)]
pub struct BounceEasing;

impl EasingFunction for BounceEasing {
    fn get(&self, pos: f32, x_range: Range<f32>, y_range: Range<f32>, mode: EasingMode) -> f32 {
        let t = (pos - x_range.start) / (x_range.end - x_range.start);
        let value = match mode {
            EasingMode::In => 1.0 - bounce_out(1.0 - t),
            EasingMode::Out => bounce_out(t),
            EasingMode::InOut => {
                if t < 0.5 {
                    0.5 * (1.0 - bounce_out(1.0 - 2.0 * t))
                } else {
                    0.5 * bounce_out(2.0 * t - 1.0) + 0.5
                }
            }
        };
        y_range.start + value * (y_range.end - y_range.start)
    }
}

fn bounce_out(t: f32) -> f32 {
    if t < 1.0 / 2.75 {
        7.5625 * t * t
    } else if t < 2.0 / 2.75 {
        let t = t - 1.5 / 2.75;
        7.5625 * t * t + 0.75
    } else if t < 2.5 / 2.75 {
        let t = t - 2.25 / 2.75;
        7.5625 * t * t + 0.9375
    } else {
        let t = t - 2.625 / 2.75;
        7.5625 * t * t + 0.984375
    }
}

// Elastic Easing
#[derive(Clone)]
pub struct ElasticEasing;

impl EasingFunction for ElasticEasing {
    fn get(&self, pos: f32, x_range: Range<f32>, y_range: Range<f32>, mode: EasingMode) -> f32 {
        let t = (pos - x_range.start) / (x_range.end - x_range.start);
        let value = match mode {
            EasingMode::In => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    -2f32.powf(10.0 * (t - 1.0))
                        * (2.0 * std::f32::consts::PI * (t - 1.1) * 2.0).sin()
                }
            }
            EasingMode::Out => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    2f32.powf(-10.0 * t) * (2.0 * std::f32::consts::PI * (t - 0.1) * 2.0).sin()
                        + 1.0
                }
            }
            EasingMode::InOut => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    -0.5 * 2f32.powf(20.0 * t - 10.0)
                        * (2.0 * std::f32::consts::PI * (t - 0.1125) * 2.0).sin()
                } else {
                    2f32.powf(-20.0 * t + 10.0)
                        * (2.0 * std::f32::consts::PI * (t - 0.1125) * 2.0).sin()
                        * 0.5
                        + 1.0
                }
            }
        };
        y_range.start + value * (y_range.end - y_range.start)
    }
}
