use crate::drawable::DrawableCallbacks;
use crate::styles::{Dimension, Location};
use mvutils::utils::{Map, PClamp, Percentage};
use num_traits::AsPrimitive;
use mvcore::color::{ColorFormat, RgbColor};
use mve2d::renderer2d::GameRenderer2D;

pub struct ColorDrawable {
    pub color: RgbColor,
}

impl ColorDrawable {
    pub fn new(color: RgbColor) -> Self {
        Self { color }
    }
}

impl DrawableCallbacks for ColorDrawable {
    fn draw(&mut self, location: Location<i32>, renderer: &mut GameRenderer2D) {
        todo!("implement this")
    }
}

pub enum GradientType {
    Linear(f32),
    Radial,
}

pub(crate) struct GradientMarker {
    pub(crate) color: RgbColor,
    pub(crate) percentage: f32,
}

pub struct GradientDrawable {
    pub start_color: RgbColor,
    pub end_color: RgbColor,
    pub markers: Vec<GradientMarker>,
    pub gradient_type: GradientType,
}

impl GradientDrawable {
    pub fn new(gradient_type: GradientType, start_color: RgbColor, end_color: RgbColor) -> Self {
        Self {
            start_color,
            end_color,
            markers: vec![],
            gradient_type,
        }
    }

    pub fn add_marker(&mut self, marker: GradientMarker) {
        self.markers.push(marker);
    }
}

impl DrawableCallbacks for GradientDrawable {
    fn draw(&mut self, location: Location<i32>, renderer2d: &mut GameRenderer2D) {
        match self.gradient_type {
            GradientType::Linear(angle) => for marker in &self.markers {},
            GradientType::Radial => {}
        }
    }
}

pub struct SimpleGradientDrawable {
    pub start_color: RgbColor,
    pub end_color: RgbColor,
    pub markers: Vec<GradientMarker>,
    pub gradient_type: GradientType,
}

impl SimpleGradientDrawable {
    pub fn new(gradient_type: GradientType, start_color: RgbColor, end_color: RgbColor) -> Self {
        Self {
            start_color,
            end_color,
            markers: vec![],
            gradient_type,
        }
    }

    pub fn add_marker(&mut self, percentage: f32, color: RgbColor) {
        self.markers.push(GradientMarker { color, percentage });
    }
}

impl DrawableCallbacks for SimpleGradientDrawable {
    fn draw(&mut self, location: Location<i32>, renderer: &mut GameRenderer2D) {
        todo!("implement this")
    }
}
