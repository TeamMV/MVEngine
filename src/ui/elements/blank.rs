use crate::ui::attributes::Attributes;
use crate::ui::context::UiContext;
use crate::ui::elements::child::Child;
use crate::ui::elements::{UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use crate::ui::rendering::ctx::DrawContext2D;
use crate::ui::styles::{Dimension, UiStyle};

#[derive(Clone)]
pub struct Blank {
    children: Vec<Child>,
}

impl UiElementCallbacks for Blank {
    fn draw(&mut self, ctx: &mut DrawContext2D) {}
}

impl UiElementStub for Blank {
    fn new(_: UiContext, _: Attributes, _: UiStyle) -> Self
    where
        Self: Sized,
    {
        Self { children: vec![] }
    }

    fn wrap(self) -> UiElement {
        UiElement::Blank(self)
    }

    fn attributes(&self) -> &Attributes {
        unreachable!()
    }

    fn attributes_mut(&mut self) -> &mut Attributes {
        unreachable!()
    }

    fn state(&self) -> &UiElementState {
        unreachable!()
    }

    fn state_mut(&mut self) -> &mut UiElementState {
        unreachable!()
    }

    fn style(&self) -> &UiStyle {
        unreachable!()
    }

    fn style_mut(&mut self) -> &mut UiStyle {
        unreachable!()
    }

    fn components(&self) -> (&Attributes, &UiStyle, &UiElementState) {
        unreachable!()
    }

    fn components_mut(&mut self) -> (&mut Attributes, &mut UiStyle, &mut UiElementState) {
        unreachable!()
    }

    fn add_child(&mut self, child: Child) {
        self.children.push(child);
    }

    fn children(&self) -> &[Child] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut [Child] {
        &mut self.children
    }

    fn get_size(&self, _: &str) -> Dimension<i32> {
        unreachable!()
    }
}

impl Blank {
    fn decompose(self) -> Vec<Child> {
        self.children
    }
}
