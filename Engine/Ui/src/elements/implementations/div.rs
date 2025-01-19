use mvutils::unsafe_utils::Unsafe;
use mvcore::color::RgbColor;
use crate::attributes::Attributes;
use crate::context::UiContext;
use crate::elements::{UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use crate::elements::child::Child;
use crate::elements::components::ElementBody;
use crate::render::ctx;
use crate::render::ctx::DrawContext2D;
use crate::styles::{Dimension, UiStyle};

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
        let rect = &self.state.rect;
        //println!("xywh: {},{},{},{}", rect.x(), rect.y(), rect.width(), rect.height());
        let this = unsafe { Unsafe::cast_static(self) };
        self.body.draw(this, ctx);
        //ctx.shape(ctx::rectangle()
        //    .xywh(rect.x(), rect.y(), rect.width(), rect.height())
        //    .color(RgbColor::red().alpha(127))
        //    .create()
        //);
        for children in &self.state.children {
            match children {
                Child::String(_) => {}
                Child::Element(e) => {
                    let mut guard = e.write();
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
            body: ElementBody::new(context),
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
