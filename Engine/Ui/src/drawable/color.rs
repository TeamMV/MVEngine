use crate::drawable::{DrawableCallbacks, UiDrawableTransformations};
use crate::styles::{Dimension, Location};
use mvcore::color::{ColorFormat, RgbColor};
use mve2d::renderer2d::GameRenderer2D;
use mvutils::utils::{Map, PClamp, Percentage};
use num_traits::AsPrimitive;
use crate::elements::UiElementState;

pub struct ColorDrawable {
    pub color: RgbColor,
}

impl ColorDrawable {
    pub fn new(color: RgbColor) -> Self {
        Self { color }
    }
}

impl DrawableCallbacks for ColorDrawable {
    fn draw(&mut self, computed: &UiElementState, transformations: UiDrawableTransformations) {
        let origin = &computed.transforms.origin;
        let x = computed.rect.x;
        let y = computed.rect.y;

        let width = computed.rect.width;
        let height = computed.rect.height;

        let ox = origin.get_actual_x(x, width, computed);
        let oy = origin.get_actual_y(y, height, computed);

        let rotation = computed.transforms.rotation + transformations.rotation;

        let x = computed.rect.x + computed.transforms.translation.width;
        let y = computed.rect.y + computed.transforms.translation.height;
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
    fn draw(&mut self, computed: &UiElementState, transformations: UiDrawableTransformations) {
        todo!()
    }
}