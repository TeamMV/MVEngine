use crate::attributes::Attributes;
use crate::context::UiContext;
use crate::elements::{UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use crate::elements::components::ElementBody;
use crate::render::ctx::DrawContext2D;
use crate::styles::{Dimension, UiStyle};

pub struct Button {
    state: UiElementState,
    style: UiStyle,
    attributes: Attributes,
    //body: ElementBody
}

impl UiElementCallbacks for Button {
    fn draw(&mut self, ctx: &mut DrawContext2D) {
        todo!()
    }
}

impl UiElementStub for Button {
    fn new(context: UiContext, attributes: Attributes, style: UiStyle) -> Self
    where
        Self: Sized
    {
        Self {
            state: UiElementState::new(),
            style,
            attributes,
        }
    }

    fn wrap(self) -> UiElement {
        UiElement::Button(self)
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