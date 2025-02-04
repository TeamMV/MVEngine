use mvutils::unsafe_utils::Unsafe;
use crate::ui::attributes::Attributes;
use crate::ui::context::UiContext;
use crate::ui::elements::{UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use crate::ui::elements::child::Child;
use crate::ui::elements::components::ElementBody;
use crate::ui::rendering::ctx::DrawContext2D;
use crate::ui::styles::{Dimension, UiStyle};

#[derive(Clone)]
pub struct Div {
    context: UiContext,
    attributes: Attributes,
    style: UiStyle,
    state: UiElementState,
    body: ElementBody<Div>
}

impl UiElementCallbacks for Div {
    fn draw(&mut self, ctx: &mut DrawContext2D) {
        let this = unsafe { Unsafe::cast_static(self) };
        self.body.draw(this, ctx, &self.context);
        for children in &self.state.children {
            match children {
                Child::String(_) => {}
                Child::Element(e) => {
                    let mut guard = e.get_mut();
                    guard.draw(ctx);
                }
                Child::State(_) => {}
            }
        }
    }
}

impl UiElementStub for Div {
    fn new(context: UiContext, attributes: Attributes, style: UiStyle) -> Self
    where
        Self: Sized,
    {
        Self {
            context: context.clone(),
            attributes,
            style,
            state: UiElementState::new(),
            body: ElementBody::new(),
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
