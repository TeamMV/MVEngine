use crate::input::{Input, RawInputEvent};
use crate::rendering::pipeline::RenderingPipeline;
use crate::rendering::OpenGLRenderer;
use crate::ui::attributes::Attributes;
use crate::ui::context::UiContext;
use crate::ui::elements::child::Child;
use crate::ui::elements::components::boring::BoringText;
use crate::ui::elements::components::ElementBody;
use crate::ui::elements::{create_style_obs, Element, LocalElement, UiElement, UiElementBuilder, UiElementCallbacks, UiElementState, UiElementStub, _Self};
use crate::ui::geometry::SimpleRect;
use crate::ui::styles::{UiStyle, UiStyleWriteObserver};
use mvutils::enum_val_ref_mut;
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use std::ops::Deref;
use std::rc::{Rc, Weak};

#[derive(Clone)]
pub struct Button {
    rc: LocalElement,

    context: UiContext,
    state: UiElementState,
    style: UiStyle,
    attributes: Attributes,
    body: ElementBody,
    text_body: BoringText,
}

impl UiElementCallbacks for Button {
    fn draw(&mut self, ctx: &mut RenderingPipeline<OpenGLRenderer>, crop_area: &SimpleRect, debug: bool) {
        self.body.draw(&self.style, &self.state, ctx, &self.context, crop_area);
        let inner_crop = crop_area.create_intersection(&self.state.content_rect.bounding);
        for children in &self.state.children {
            match children {
                Child::String(s) => {
                    self.text_body.draw(0, 0, s, &self.state, &self.style, &self.body, ctx, &self.context, crop_area);
                }
                Child::Element(e) => {
                    let guard = e.inner.get_mut();
                    guard.frame_callback(ctx, &inner_crop, debug);
                }
                Child::State(s) => {
                    let guard = s.read();
                    let s = guard.deref();
                    self.text_body.draw(0, 0, s, &self.state, &self.style, &self.body, ctx, &self.context, crop_area);
                }
                _ => {}
            }
        }
        self.body.draw_scrollbars(&self.style, &self.state, ctx, &self.context, crop_area);
    }
}

impl UiElementBuilder for Button {
    fn _builder(&self, context: UiContext, attributes: Attributes, style: UiStyle) -> _Self {
        ()
    }

    fn set_weak(&mut self, weak: LocalElement) {
        self.rc = weak;
    }

    fn wrap(self) -> UiElement {
        UiElement::Button(self)
    }
}

impl Button {
    pub fn builder(context: UiContext, attributes: Attributes, style: UiStyle) -> Self {
        Self {
            rc: LocalElement::new(),
            context: context.clone(),
            state: UiElementState::new(context),
            style: style.clone(),
            attributes,
            body: ElementBody::new(),
            text_body: BoringText,
        }
    }
}

impl UiElementStub for Button {
    fn wrapped(&self) -> Element {
        self.rc.to_wrapped()
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

    fn style_mut(&mut self) -> UiStyleWriteObserver {
        create_style_obs(&mut self.style, &mut self.state)
    }

    fn context(&self) -> &UiContext {
        &self.context
    }

    fn body(&self) -> &ElementBody {
        &self.body
    }

    fn body_mut(&mut self) -> &mut ElementBody {
        &mut self.body
    }
}
