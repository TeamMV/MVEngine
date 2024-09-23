use crate::drawable::{DrawableCallbacks, UiDrawable, UiDrawableTransformations};
use crate::elements::UiElementState;

pub struct LayerDrawable {
    inner: Vec<UiDrawable>
}

impl LayerDrawable {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }

    pub fn add_inner(&mut self, inner: UiDrawable) {
        self.inner.push(inner);
    }
}

impl DrawableCallbacks for LayerDrawable {
    fn draw(&mut self, computed: &UiElementState, transformations: UiDrawableTransformations) {
        for d in &mut self.inner {
            d.draw(computed, transformations.clone());
        }
    }
}