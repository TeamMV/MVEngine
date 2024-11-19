use std::num::ParseIntError;
use hashbrown::HashMap;
use crate::drawable::{DrawableCallbacks, DrawableCreate, UiDrawable, UiDrawableTransformations};
use crate::elements::UiElementState;
use crate::styles::Origin;
use crate::utils;

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

impl DrawableCallbacks for PaddedDrawable {
    fn draw(&mut self, computed: &UiElementState, transformations: UiDrawableTransformations) {
        self.inner.draw(computed, transformations.modify(|t| {
            t.translation.0 += self.paddings[1];
            t.translation.1 += self.paddings[2];
            t.shrink.0 += self.paddings[0] + self.paddings[1];
            t.shrink.1 += self.paddings[3] + self.paddings[2];
        }));
    }
}

impl DrawableCreate for PaddedDrawable {
    fn create(inner: Vec<UiDrawable>, attributes: HashMap<String, String>) -> Result<UiDrawable, String> {
        if inner.len() != 1 {
            return Err(String::from("PaddedDrawable must have exactly one Drawable"));
        }
        let pad_str = attributes.get("padding");
        let padding = if pad_str.is_none() {
            [0; 4]
        } else {
            crate::parse::parse_4xi32(pad_str.unwrap())?
        };
        Ok(UiDrawable::Padded(PaddedDrawable::new(padding, inner.into_iter().next().unwrap())))
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

impl DrawableCreate for RotateDrawable {
    fn create(inner: Vec<UiDrawable>, attributes: HashMap<String, String>) -> Result<UiDrawable, String> {
        if inner.len() != 1 {
            return Err(String::from("RotateDrawable must have exactly one Drawable"));
        }
        let rot_str = attributes.get("rotation");
        let rotation = if rot_str.is_none() {
            0f32
        } else {
            crate::parse::parse_angle(rot_str.unwrap())?
        };
        let origin_str = attributes.get("origin");
        let origin = if origin_str.is_none() {
            None
        } else {
            Some(crate::parse::parse_origin(origin_str.unwrap())?)
        };
        Ok(UiDrawable::Rotate(RotateDrawable::new(rotation, origin, inner.into_iter().next().unwrap())))
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

impl DrawableCreate for TranslateDrawable {
    fn create(inner: Vec<UiDrawable>, attributes: HashMap<String, String>) -> Result<UiDrawable, String> {
        if inner.len() != 1 {
            return Err(String::from("TranslateDrawable must have exactly one Drawable"));
        }
        let tx_str = attributes.get("x");
        let tx = if tx_str.is_none() {
            0
        } else {
            tx_str.unwrap().parse::<i32>().map_err(ParseIntError::to_string)?
        };
        let ty_str = attributes.get("y");
        let ty = if ty_str.is_none() {
            0
        } else {
            ty_str.unwrap().parse::<i32>().map_err(ParseIntError::to_string)?
        };
        Ok(UiDrawable::Translate(TranslateDrawable::new(tx, ty, inner.into_iter().next().unwrap())))
    }
}