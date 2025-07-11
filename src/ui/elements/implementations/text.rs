use crate::input::{Input, RawInputEvent};
use crate::rendering::{OpenGLRenderer, RenderContext};
use crate::ui::attributes::Attributes;
use crate::ui::context::UiContext;
use crate::ui::elements::child::Child;
use crate::ui::elements::components::ElementBody;
use crate::ui::elements::components::text::TextBody;
use crate::ui::elements::{create_style_obs, Element, UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use crate::ui::geometry::SimpleRect;
use crate::ui::rendering::UiRenderer;
use crate::ui::styles::{UiStyle, UiStyleWriteObserver};
use mvutils::enum_val_ref_mut;
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use std::ops::Deref;
use std::rc::{Rc, Weak};
use ropey::Rope;
use crate::rendering::pipeline::RenderingPipeline;

#[derive(Clone)]
pub struct Text {
    rc: Weak<DangerousCell<UiElement>>,

    context: UiContext,
    state: UiElementState,
    style: UiStyle,
    attributes: Attributes,

    //Needed but not used
    body: ElementBody,
    text: TextBody,
}

impl Text {
    fn draw_string(&mut self, s: &Rope, ctx: &mut impl RenderContext, crop_area: &SimpleRect) {
        let this = unsafe { Unsafe::cast_lifetime_mut(self) };
        self.text.draw(this, s, ctx, crop_area);
    }
}

impl UiElementCallbacks for Text {
    fn draw(&mut self, ctx: &mut RenderingPipeline<OpenGLRenderer>, crop_area: &SimpleRect, debug: bool) {
        let this = unsafe { Unsafe::cast_lifetime_mut(self) };
        for children in &self.state.children {
            match children {
                Child::String(s) => {
                    this.draw_string(s, ctx, crop_area);
                }
                Child::State(s) => {
                    let guard = s.read();
                    let s = guard.deref();
                    this.draw_string(s, ctx, crop_area);
                }
                _ => {}
            }
        }
        self.body.draw_scrollbars(this, ctx, &self.context, crop_area);
    }

    fn raw_input_callback(&mut self, _action: RawInputEvent, _input: &Input) -> bool {
        false
    }
}

impl UiElementStub for Text {
    fn new(context: UiContext, attributes: Attributes, style: UiStyle) -> Element
    where
        Self: Sized,
    {
        let this = Self {
            rc: Weak::new(),
            context: context.clone(),
            state: UiElementState::new(context),
            style: style.clone(),
            attributes,
            body: ElementBody::new(),
            text: TextBody {},
        };
        let rc = Rc::new(DangerousCell::new(this.wrap()));
        let e = rc.get_mut();
        let txt = enum_val_ref_mut!(UiElement, e, Text);
        txt.rc = Rc::downgrade(&rc);

        rc
    }

    fn wrap(self) -> UiElement {
        UiElement::Text(self)
    }

    fn wrapped(&self) -> Element {
        self.rc.upgrade().expect("Reference to this self")
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
