use std::rc::{Rc, Weak};
use mvutils::enum_val_ref_mut;
use crate::ui::attributes::Attributes;
use crate::ui::context::UiContext;
use crate::ui::elements::child::Child;
use crate::ui::elements::components::ElementBody;
use crate::ui::elements::{Element, UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use crate::ui::rendering::ctx::DrawContext2D;
use crate::ui::styles::UiStyle;
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use crate::input::{Input, RawInputEvent};
use crate::ui::styles::types::Dimension;

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
    fn draw(&mut self, ctx: &mut DrawContext2D) {
        let this = unsafe { Unsafe::cast_static(self) };
        self.body.draw(this, ctx, &self.context);
        for children in &self.state.children {
            match children {
                Child::String(_) => {}
                Child::Element(e) => {
                    let guard = e.get_mut();
                    guard.draw(ctx);
                }
                Child::State(_) => {}
            }
        }
    }

    fn raw_input(&mut self, action: RawInputEvent, input: &Input) -> bool {
        let unsafe_self = unsafe { Unsafe::cast_mut_static(self) };
        self.body.on_input(unsafe_self, action.clone(), input);
        
        for elem in &self.state.children {
            if let Child::Element(child) = elem {
                let child = child.get_mut();
                child.raw_input(action.clone(), input);
            }
        }
        true
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
            state: UiElementState::new(),
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

    fn body(&self) -> &ElementBody {
        &self.body
    }

    fn body_mut(&mut self) -> &mut ElementBody {
        &mut self.body
    }
}
