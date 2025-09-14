use crate::input::consts::MouseButton;
use crate::input::{Input, MouseAction, RawInputEvent};
use crate::rendering::pipeline::RenderingPipeline;
use crate::rendering::OpenGLRenderer;
use crate::ui::attributes::{Attributes, IntoAttrib};
use crate::ui::context::UiContext;
use crate::ui::elements::child::Child;
use crate::ui::elements::components::boring::BoringText;
use crate::ui::elements::components::ElementBody;
use crate::ui::elements::{create_style_obs, Element, LocalElement, UiElement, UiElementBuilder, UiElementCallbacks, UiElementState, UiElementStub, _Self};
use crate::ui::geometry::{shape, SimpleRect};
use crate::ui::rendering::WideRenderContext;
use crate::ui::styles::{UiStyle, UiStyleWriteObserver};
use mvutils::enum_val_ref_mut;
use mvutils::state::State;
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use ropey::Rope;
use std::rc::{Rc, Weak};

#[derive(Clone)]
pub struct CheckBox {
    rc: LocalElement,
    context: UiContext,
    state: UiElementState,
    style: UiStyle,
    attributes: Attributes,
    body: ElementBody,
    text_body: BoringText,
    selected: State<bool>
}

impl UiElementCallbacks for CheckBox {
    fn draw(&mut self, ctx: &mut RenderingPipeline<OpenGLRenderer>, crop_area: &SimpleRect, debug: bool) {
        self.body.draw_height_square(&self.style, &self.state, ctx, &self.context, crop_area);
        let inner_crop = crop_area.create_intersection(&self.state.content_rect.bounding);
        let text_w = if let Some(child) = self.state.children.first() {
            match child {
                Child::String(s) => {
                    self.draw_text(s, ctx, &inner_crop)
                }
                Child::State(s) => {
                    let s = s.read();
                    self.draw_text(&s, ctx, &inner_crop)
                }
                _ => { 0 }
            }
        } else { 0 };
        self.state.requested_width = Some(text_w);
        if *self.selected.read() {
            let cr = &self.state.content_rect;
            let rect = SimpleRect::new(cr.x(), cr.y(), cr.height(), cr.height());
            shape::utils::draw_shape_style_at(ctx, &self.context, &rect, &self.style.detail, &self.state, &self.body, |s| &s.detail, Some(crop_area.clone()));
        }
        self.body.draw_scrollbars(&self.style, &self.state, ctx, &self.context, crop_area);
    }

    fn raw_input_callback(&mut self, action: RawInputEvent, input: &Input) -> bool {
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
    fn draw_text(&self, s: &Rope, ctx: &mut impl WideRenderContext, crop: &SimpleRect) -> i32 {
        let height = self.state.rect.height();
        self.text_body.draw(height, 0, s, &self.state, &self.style, &self.body, ctx, &self.context, crop) + height
    }
}

impl UiElementBuilder for CheckBox {
    fn _builder(&self, context: UiContext, attributes: Attributes, style: UiStyle) -> _Self {
        ()
    }

    fn set_weak(&mut self, weak: LocalElement) {
        self.rc = weak;
    }

    fn wrap(self) -> UiElement {
        UiElement::CheckBox(self)
    }
}

impl CheckBox {
    pub fn builder(context: UiContext, attributes: Attributes, style: UiStyle) -> Self {
        Self {
            rc: LocalElement::new(),
            context: context.clone(),
            state: UiElementState::new(context),
            style: style.clone(),
            attributes,
            body: ElementBody::new(),
            text_body: BoringText,
            selected: State::new(false),
        }
    }
    
    pub fn selected<T: IntoAttrib<State<bool>>>(mut self, attrib: T) -> Self {
        self.selected = attrib.into_attrib();
        self
    }
}

impl UiElementStub for CheckBox {
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