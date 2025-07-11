use crate::input::{Input, RawInputEvent};
use crate::rendering::{OpenGLRenderer, RenderContext};
use crate::ui::attributes::Attributes;
use crate::ui::context::UiContext;
use crate::ui::elements::child::Child;
use crate::ui::elements::components::ElementBody;
use crate::ui::elements::components::scroll::ScrollBars;
use crate::ui::elements::{create_style_obs, Element, UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use crate::ui::geometry::SimpleRect;
use crate::ui::rendering::UiRenderer;
use crate::ui::styles::{UiStyle, UiStyleWriteObserver};
use mvutils::enum_val_ref_mut;
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use std::rc::{Rc, Weak};
use crate::rendering::pipeline::RenderingPipeline;

#[derive(Clone)]
pub struct Div {
    rc: Weak<DangerousCell<UiElement>>,

    context: UiContext,
    attributes: Attributes,
    style: UiStyle,
    state: UiElementState,
    body: ElementBody,
}

impl UiElementCallbacks for Div {
    fn draw(&mut self, ctx: &mut RenderingPipeline<OpenGLRenderer>, crop_area: &SimpleRect, debug: bool) {
        let this = unsafe { Unsafe::cast_lifetime(self) };
        self.body.draw(this, ctx, &self.context, crop_area);
        let inner_crop = crop_area.create_intersection(&self.state.content_rect.bounding);
        for children in &self.state.children {
            match children {
                Child::String(_) => {}
                Child::Element(e) => {
                    let guard = e.get_mut();
                    guard.frame_callback(
                        ctx,
                        &inner_crop,
                        debug
                    );
                }
                Child::State(_) => {}
                _ => {}
            }
        }
        self.body.draw_scrollbars(this, ctx, &self.context, crop_area);
    }
}

impl UiElementStub for Div {
    fn new(context: UiContext, attributes: Attributes, style: UiStyle) -> Element
    where
        Self: Sized,
    {
        let this = Self {
            rc: Weak::new(),
            context: context.clone(),
            attributes,
            style,
            state: UiElementState::new(context),
            body: ElementBody::new(),
        };

        let rc = Rc::new(DangerousCell::new(this.wrap()));
        let e = rc.get_mut();
        let div = enum_val_ref_mut!(UiElement, e, Div);
        div.rc = Rc::downgrade(&rc);

        rc
    }

    fn wrap(self) -> UiElement {
        UiElement::Div(self)
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
