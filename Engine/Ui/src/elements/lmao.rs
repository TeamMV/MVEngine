use std::sync::Arc;
use parking_lot::RwLock;
use mvcore::color::RgbColor;
use mvcore::math::vec::{Vec2, Vec3};
use mvcore::render::renderer::Renderer;
use mve2d::renderer2d::{Renderer2D, Shape};
use crate::{resolve};
use crate::attributes::Attributes;
use crate::elements::child::Child;
use crate::elements::{UiElementStub, UiElementCallbacks, UiElementState};
use crate::styles::{Dimension, UiStyle};

pub struct LmaoElement {
    state: UiElementState,
    style: UiStyle,
    col: RgbColor,
    attributes: Attributes,
}

impl UiElementCallbacks for LmaoElement {
    fn init(&mut self) {
        todo!()
    }

    fn draw(&mut self, renderer: &mut Renderer2D) {
        let shape = Shape::Rectangle {
            position: Vec3::new(self.state.x as f32, renderer.get_extent().height as f32 - self.state.y as f32 - self.state.height as f32, 0f32),
            rotation: Default::default(),
            scale: Vec2::new(self.state.width as f32, self.state.height as f32),
            tex_id: None,
            tex_coord: Default::default(),
            color: self.col.as_vec4(),
            blending: 0.0,
        };

        renderer.add_shape(shape);
    }
}

impl UiElementStub for LmaoElement {
    fn new(attributes: Attributes, style: UiStyle) -> Self
    where
        Self: Sized,
    {
        Self {
            state: UiElementState::new(),
            style,
            col: RgbColor::blue(),
            attributes,
        }
    }

    fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    fn attributes_mut(&mut self) -> &mut Attributes {
        &mut self.attributes
    }

    fn state(&self) -> &UiElementState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut UiElementState {
        &mut self.state
    }

    fn style(&self) -> &UiStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut UiStyle {
        &mut self.style
    }

    fn components(&self) -> (&Attributes, &UiStyle, &UiElementState) {
        (&self.attributes, &self.style, &self.state)
    }

    fn components_mut(&mut self) -> (&mut Attributes, &mut UiStyle, &mut UiElementState) {
        (&mut self.attributes, &mut self.style, &mut self.state)
    }

    fn get_size(&self, s: &str) -> Dimension<i32> {
        todo!("get font and calc size");
    }
}