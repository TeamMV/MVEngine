use crate::rendering::pipeline::RenderingPipeline;
use crate::rendering::OpenGLRenderer;
use crate::ui::attributes::Attributes;
use crate::ui::context::UiContext;
use crate::ui::elements::child::Child;
use crate::ui::elements::components::boring::BoringText;
use crate::ui::elements::components::ElementBody;
use crate::ui::elements::{create_style_obs, Element, UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use crate::ui::geometry::{shape, SimpleRect};
use crate::ui::rendering::WideRenderContext;
use crate::ui::styles::{UiStyle, UiStyleWriteObserver};
use mvutils::enum_val_ref_mut;
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use std::rc::{Rc, Weak};
use mvutils::state::State;
use crate::input::{Input, MouseAction, RawInputEvent};
use crate::input::consts::MouseButton;

#[derive(Clone)]
pub struct CheckBox {
    rc: Weak<DangerousCell<UiElement>>,
    context: UiContext,
    state: UiElementState,
    style: UiStyle,
    attributes: Attributes,
    body: ElementBody,
    text_body: BoringText<CheckBox>,
    selected: State<bool>
}

impl UiElementCallbacks for CheckBox {
    fn draw(&mut self, ctx: &mut RenderingPipeline<OpenGLRenderer>, crop_area: &SimpleRect) {
        let this = unsafe { Unsafe::cast_lifetime(self) };
        self.body.draw_height_square(this, ctx, &self.context, crop_area);
        let text_w = if let Some(child) = self.state.children.first() {
            match child {
                Child::String(s) => {
                    self.draw_text(s, ctx, crop_area)
                }
                Child::State(s) => {
                    let s = s.read();
                    self.draw_text(&s, ctx, crop_area)
                }
                _ => { 0 }
            }
        } else { 0 };
        self.state.requested_width = Some(text_w);
        if *self.selected.read() {
            let cr = &self.state.content_rect;
            let rect = SimpleRect::new(cr.x(), cr.y(), cr.height(), cr.height());
            shape::utils::draw_shape_style_at(ctx, &self.context, &rect, &self.style.detail, self, |s| &s.detail, Some(crop_area.clone()));
        }
    }

    fn raw_input(&mut self, action: RawInputEvent, input: &Input) -> bool {
        self.super_input(action.clone(), input);
        if let RawInputEvent::Mouse(MouseAction::Press(MouseButton::Left)) = action {
            if self.inside(input.mouse_x, input.mouse_y) {
                let current = *self.selected.read();
                let mut g = self.selected.write();
                *g = !current;
                return true;
            }
        }
        false
    }
}

impl CheckBox {
    fn draw_text(&self, s: &str, ctx: &mut impl WideRenderContext, crop: &SimpleRect) -> i32 {
        let height = self.state.rect.height();
        self.text_body.draw(height, 0, s, &self, ctx, &self.context, crop) + height
    }
}

impl UiElementStub for CheckBox {
    fn new(context: UiContext, attributes: Attributes, style: UiStyle) -> Element
    where
        Self: Sized
    {
        let selected = match attributes.attribs.get("selected") {
            None => State::new(false),
            Some(v) => v.as_bool_state(),
        };
        
        let this = Self {
            rc: Weak::new(),
            context: context.clone(),
            state: UiElementState::new(context),
            style: style.clone(),
            attributes,
            body: ElementBody::new(),
            text_body: BoringText::new(),
            selected,
        };
        let rc = Rc::new(DangerousCell::new(this.wrap()));
        let e = rc.get_mut();
        let cb = enum_val_ref_mut!(UiElement, e, CheckBox);
        cb.rc = Rc::downgrade(&rc);

        rc
    }

    fn wrap(self) -> UiElement {
        UiElement::CheckBox(self)
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