use crate::render::color::RgbColor;
use crate::render::draw2d::DrawContext2D;
use crate::ui::drawable::DrawableCallbacks;
use crate::ui::styles::{Dimension, Location};
use mvutils::utils::{Map, PClamp, Percentage};
use num_traits::AsPrimitive;

pub struct ColorDrawable {
    pub color: RgbColor,
}

impl ColorDrawable {
    pub fn new(color: RgbColor) -> Self {
        Self { color }
    }
}

impl DrawableCallbacks for ColorDrawable {
    fn draw(&mut self, location: Location<i32>, ctx: &mut DrawContext2D) {
        ctx.color(self.color);
        ctx.rectangle_origin_rotated(
            location.x,
            location.y,
            location.dimension.width,
            location.dimension.height,
            location.rotation,
            location.origin.x + location.x,
            location.origin.y + location.y,
        );
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
    fn draw(&mut self, location: Location<i32>, ctx: &mut DrawContext2D) {
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
    fn draw(&mut self, location: Location<i32>, ctx: &mut DrawContext2D) {
        match self.gradient_type {
            GradientType::Linear(angle) => {
                let mapped = angle.p_clamp(0.0, 360.0).map(&(0.0..360.0), &(0.0..3.0)) as i32;

                let x = location.x as f32;
                let y = location.y as f32;
                let w = location.dimension.width as f32;
                let h = location.dimension.height as f32;
                let rot = location.rotation;
                let rx = location.origin.x as f32;
                let ry = location.origin.y as f32;

                match mapped {
                    0 => {
                        //--->
                        ctx.begin_shape();

                        let mut prev = &GradientMarker {
                            color: self.start_color,
                            percentage: 0.0,
                        };

                        for marker in &self.markers {
                            let p = marker.percentage;
                            let prev_p = prev.percentage;

                            let x1 = x + prev_p.value(w);
                            let x2 = x + p.value(w);

                            //ctx.color(marker.color);
                            //ctx.rectangle(x1 as i32, y as i32, (x2 - x1) as i32, h as i32);

                            ctx.vertex_color(x1, y + h, rot, rx, ry, prev.color);
                            ctx.vertex_color(x1, y, rot, rx, ry, prev.color);
                            ctx.vertex_color(x2, y, rot, rx, ry, marker.color);

                            ctx.vertex_color(x2, y, rot, rx, ry, marker.color);
                            ctx.vertex_color(x2, y + h, rot, rx, ry, marker.color);
                            ctx.vertex_color(x1, y + h, rot, rx, ry, prev.color);

                            prev = marker;
                        }

                        let x1 = x + prev.percentage.value(w);
                        let x2 = x + w;
                        let marker = GradientMarker {
                            color: self.end_color,
                            percentage: 100.0,
                        };

                        ctx.vertex_color(x1, y + h, rot, rx, ry, prev.color);
                        ctx.vertex_color(x1, y, rot, rx, ry, prev.color);
                        ctx.vertex_color(x2, y, rot, rx, ry, marker.color);

                        ctx.vertex_color(x2, y, rot, rx, ry, marker.color);
                        ctx.vertex_color(x2, y + h, rot, rx, ry, marker.color);
                        ctx.vertex_color(x1, y + h, rot, rx, ry, prev.color);
                    }
                    1 => {
                        // \/
                    }
                    2 => {
                        //<---
                    }
                    3 => {
                        // /\
                    }
                    _ => unreachable!(),
                }
            }
            GradientType::Radial => {}
        }
    }
}
