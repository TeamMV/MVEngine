pub mod color;
mod positioning;
mod collection;

use crate::drawable::collection::LayerDrawable;
use crate::drawable::color::{ColorDrawable, GradientDrawable};
use crate::drawable::positioning::{PaddedDrawable, RotateDrawable, TranslateDrawable};
use crate::elements::UiElementState;
use crate::styles::Origin;

#[derive(Clone)]
pub struct UiDrawableTransformations {
    translation: (i32, i32),
    size: DrawableSize,
    origin: Origin,
    rotation: f32
}

impl UiDrawableTransformations {
    pub(crate) fn modify<F>(&self, f: F) -> UiDrawableTransformations where F: FnMut(&mut UiDrawableTransformations) {
        let mut cloned = self.clone();
        f(&mut cloned);
        cloned
    }
}

impl Default for UiDrawableTransformations {
    fn default() -> Self {
        Self {
            translation: (0, 0),
            size: DrawableSize::Scale((1.0, 1.0)),
            origin: Origin::Center,
            rotation: 0.0,
        }
    }
}

#[derive(Clone)]
enum DrawableSize {
    Fixed((i32, i32)),
    Scale((f32, f32))
}

pub enum UiDrawable {
    Color(ColorDrawable),
    Padded(PaddedDrawable),
    Gradient(GradientDrawable),
    Rotate(RotateDrawable),
    Translate(TranslateDrawable),
    Layer(LayerDrawable)
}
g
pub trait DrawableCallbacks {
    fn draw(&mut self, computed: &UiElementState, transformations: UiDrawableTransformations);
}

impl DrawableCallbacks for UiDrawable {
    fn draw(&mut self, computed: &UiElementState, transformations: UiDrawableTransformations) {
        match self {
            UiDrawable::Color(d) => d.draw(computed, transformations),
            UiDrawable::Padded(d) => d.draw(computed, transformations),
            UiDrawable::Gradient(d) => d.draw(computed, transformations),
            UiDrawable::Rotate(d) => d.draw(computed, transformations),
            UiDrawable::Translate(d) => d.draw(computed, transformations),
            UiDrawable::Layer(d) => d.draw(computed, transformations),
        }
    }
}