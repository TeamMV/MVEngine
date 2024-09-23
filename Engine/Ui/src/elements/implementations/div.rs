use crate::attributes::Attributes;
use crate::elements::{UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use crate::styles::{Dimension, UiStyle};
use mve2d::renderer2d::GameRenderer2D;

pub struct Div {
    attributes: Attributes,
    style: UiStyle,
    state: UiElementState,
}

impl UiElementCallbacks for Div {
    fn draw(&mut self, renderer: &mut GameRenderer2D) {
        todo!();
    }
}

impl UiElementStub for Div {
    fn new(attributes: Attributes, style: UiStyle) -> Self
    where
        Self: Sized,
    {
        Self {
            attributes,
            style,
            state: UiElementState::new(),
        }
    }

    fn wrap(self) -> UiElement {
        UiElement::Div(self)
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
        todo!()
    }
}
