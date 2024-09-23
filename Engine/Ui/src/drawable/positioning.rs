use crate::drawable::{DrawableCallbacks, UiDrawable, UiDrawableTransformations};
use crate::elements::UiElementState;
use crate::styles::Origin;

pub struct PaddedDrawable {
    paddings: [i32; 4],
    inner: Box<UiDrawable>
}

impl PaddedDrawable {
    pub fn new(paddings: [i32; 4], inner: UiDrawable) -> Self {
        Self { paddings, inner: Box::new(inner) }
    }

    pub fn new_splat(padding: i32, inner: UiDrawable) -> Self {
        Self {
            paddings: [padding; 4],
            inner: Box::new(inner)
        }
    }

    pub fn splat_padding(&mut self, padding: i32) {
        self.paddings = [padding; 4];
    }

    pub fn set_padding(&mut self, paddings: [i32; 4]) {
        self.paddings = paddings
    }
}

pub struct RotateDrawable {
    rotation: f32,
    inner: Box<UiDrawable>,
    custom_origin: Option<Origin>
}

impl RotateDrawable {
    pub fn new(rotation: f32, custom_origin: Option<Origin>, inner: UiDrawable) -> Self {
        Self { rotation, custom_origin, inner: Box::new(inner) }
    }
}

impl DrawableCallbacks for RotateDrawable {
    fn draw(&mut self, computed: &UiElementState, transformations: UiDrawableTransformations) {
        self.inner.draw(computed, transformations.modify(|t| {
            t.rotation += self.rotation;
            if self.custom_origin.is_some() {
                t.origin = self.custom_origin.clone().unwrap();
            }
        }));
    }
}

pub struct TranslateDrawable {
    translation_x: i32,
    translation_y: i32,
    inner: Box<UiDrawable>,
}

impl TranslateDrawable {
    pub fn new(translation_x: i32, translation_y: i32, inner: UiDrawable) -> Self {
        Self { translation_x, translation_y, inner: Box::new(inner) }
    }
}

impl DrawableCallbacks for TranslateDrawable {
    fn draw(&mut self, computed: &UiElementState, transformations: UiDrawableTransformations) {
        self.inner.draw(computed, transformations.modify(|t| {
            t.translation.0 += self.translation_x;
            t.translation.1 += self.translation_y;
        }));
    }
}