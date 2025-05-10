use crate::ui::anim::easing;
use crate::ui::attributes::Attributes;
use crate::ui::context::UiContext;
use crate::ui::ease::{Easing, EasingGen, EasingMode};
use crate::ui::elements::child::Child;
use crate::ui::elements::components::ElementBody;
use crate::ui::elements::{UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use crate::ui::rendering::ctx::DrawContext2D;
use crate::ui::styles::{Dimension, Interpolator, UiStyle};
use crate::ui::timing::{AnimationState, DurationTask, TIMING_MANAGER};
use mvutils::unsafe_utils::Unsafe;
use mvutils::utils::Percentage;
use std::ops::Deref;
use crate::input::{Input, MouseAction, RawInputEvent};
use crate::ui::elements::components::text::TextBody;

#[derive(Clone)]
pub struct Button {
    context: UiContext,
    state: UiElementState,
    style: UiStyle,
    initial_style: UiStyle,
    attributes: Attributes,
    body: ElementBody<Button>,
    text_body: TextBody<Button>,
}

impl Button {
    pub fn body(&self) -> &ElementBody<Button> {
        &self.body
    }
    
    pub fn body_mut(&mut self) -> &mut ElementBody<Button> {
        &mut self.body
    }
}

impl UiElementCallbacks for Button {
    fn draw(&mut self, ctx: &mut DrawContext2D) {
        let this = unsafe { Unsafe::cast_static(self) };
        self.body.draw(this, ctx, &self.context);
        for children in &self.state.children {
            match children {
                Child::String(s) => {
                    self.text_body.draw(s, this, ctx, &self.context);
                }
                Child::Element(e) => {
                    let mut guard = e.get_mut();
                    guard.draw(ctx);
                }
                Child::State(s) => {
                    let guard = s.read();
                    let s = guard.deref();
                    self.text_body.draw(s, this, ctx, &self.context);
                }
            }
        }
    }

    fn raw_input(&mut self, action: RawInputEvent, input: &Input) -> bool {
        let unsafe_self = unsafe { Unsafe::cast_mut_static(self) };
        self.body.on_input(unsafe_self, action, input);
        true
    }
}

impl UiElementStub for Button {
    fn new(context: UiContext, attributes: Attributes, style: UiStyle) -> Self
    where
        Self: Sized,
    {        
        let mut this = Self {
            context: context.clone(),
            state: UiElementState::new(),
            style: style.clone(),
            initial_style: style.clone(),
            attributes,
            body: ElementBody::new(),
            text_body: TextBody::new(),
        };
        
        this
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

    fn context(&self) -> &UiContext {
        &self.context
    }

    fn get_size(&self, s: &str) -> Dimension<i32> {
        todo!()
    }
}
