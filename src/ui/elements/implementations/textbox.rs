use crate::input::consts::{Key, MouseButton};
use crate::input::registry::RawInput;
use crate::input::{Input, KeyboardAction, MouseAction, RawInputEvent};
use crate::rendering::{OpenGLRenderer, RenderContext};
use crate::ui::attributes::{Attributes, IntoAttrib, UiState};
use crate::ui::context::UiContext;
use crate::ui::elements::child::Child;
use crate::ui::elements::components::ElementBody;
use crate::ui::elements::components::boring::BoringText;
use crate::ui::elements::components::edittext::EditableTextHelper;
use crate::ui::elements::{create_style_obs, Element, LocalElement, UiElement, UiElementBuilder, UiElementCallbacks, UiElementState, UiElementStub, _Self};
use crate::ui::geometry::SimpleRect;
use crate::ui::rendering::UiRenderer;
use crate::ui::styles::{UiStyle, UiStyleWriteObserver};
use crate::ui::styles::types::Dimension;
use mvutils::enum_val_ref_mut;
use mvutils::state::State;
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use std::rc::{Rc, Weak};
use ropey::Rope;
use crate::rendering::pipeline::RenderingPipeline;
use crate::utils::RopeFns;

#[derive(Clone)]
pub struct TextBox {
    rc: LocalElement,

    context: UiContext,
    state: UiElementState,
    style: UiStyle,
    attributes: Attributes,
    body: ElementBody,
    content: UiState,
    placeholder: UiState,
    focused: bool,
    helper: EditableTextHelper,
}

impl TextBox {
    pub fn get_content(&self) -> UiState {
        self.content.clone()
    }

    pub fn get_placeholder(&self) -> UiState {
        self.placeholder.clone()
    }

    pub fn focus_now(&mut self) {
        self.focused = true;
    }
}

impl UiElementCallbacks for TextBox {
    fn draw(&mut self, ctx: &mut RenderingPipeline<OpenGLRenderer>, crop_area: &SimpleRect, debug: bool) {
        self.body.draw(&self.style, &self.state, ctx, &self.context, crop_area);
        let inner_crop = crop_area.create_intersection(&self.state.content_rect.bounding);
        for children in &mut self.state.children {
            match children {
                Child::Element(e) => {
                    let guard = e.get_mut();
                    guard.frame_callback(ctx, &inner_crop, debug);
                }
                _ => {}
            }
        }
        let s = self.content.read();
        if s.is_empty() {
            if !self.focused {
                let placeholder = self.placeholder.read();
                self.helper.draw_other(&*placeholder, &self.style, &self.state, &self.body, ctx, &self.context, crop_area);
            } else {
                self.helper.draw(&self.style, &self.state, &self.body, ctx, &self.context, crop_area, true);
            }
        } else {
            if self.focused {
                self.helper.draw(&self.style, &self.state, &self.body, ctx, &self.context, crop_area, true);
            } else {
                self.helper.draw(&self.style, &self.state, &self.body, ctx, &self.context, crop_area, false);
            }
        }
        self.body.draw_scrollbars(&self.style, &self.state, ctx, &self.context, crop_area);
    }

    fn raw_input_callback(&mut self, action: RawInputEvent, input: &Input) -> bool {
        match action {
            RawInputEvent::Keyboard(ka) => {
                if self.focused {
                    match ka {
                        KeyboardAction::Release(_) => {}
                        KeyboardAction::Type(key) | KeyboardAction::Press(key) => {
                            let is_shift = input
                                .action_processor()
                                .is_raw_input(RawInput::KeyPress(Key::LShift));
                            if let Key::Back = key {
                                self.helper.backspace();
                            }
                            if let Key::Left = key {
                                self.helper.move_left(is_shift);
                            }
                            if let Key::Right = key {
                                self.helper.move_right(is_shift);
                            }
                            if let Key::End = key {
                                self.helper.move_to_end(is_shift);
                            }
                            if let Key::Home = key {
                                self.helper.move_to_start(is_shift);
                            }
                        }
                        KeyboardAction::Char(ch) => {
                            if !ch.is_control() {
                                self.helper.add_str(&ch.to_string());
                            }
                        }
                    }
                }
            }
            RawInputEvent::Mouse(ma) => match ma {
                MouseAction::Press(p) => {
                    let mx = input.mouse_x;
                    let my = input.mouse_y;
                    if let MouseButton::Left = p {
                        if self.inside(mx, my) {
                            self.focused = true;
                            return true;
                        } else {
                            self.focused = false;
                        }
                    }
                }
                _ => {}
            },
        };

        false
    }
}

impl UiElementBuilder for TextBox {
    fn _builder(&self, context: UiContext, attributes: Attributes, style: UiStyle) -> _Self {
        ()
    }

    fn set_weak(&mut self, weak: LocalElement) {
        self.rc = weak;
    }

    fn wrap(self) -> UiElement {
        UiElement::TextBox(self)
    }
}

impl TextBox {
    pub fn builder(context: UiContext, attributes: Attributes, style: UiStyle) -> Self {
        let content = State::new(Rope::new()).map_identity();
        Self {
            rc: LocalElement::new(),
            context: context.clone(),
            state: UiElementState::new(context),
            style,
            attributes,
            body: ElementBody::new(),
            content: content.clone(),
            placeholder: State::new(Rope::new()).map_identity(),
            focused: false,
            helper: EditableTextHelper::new(content),
        }
    }
    
    pub fn content<T: IntoAttrib<UiState>>(mut self, attrib: T) -> Self {
        self.content = attrib.into_attrib();
        self.helper.set_content(self.content.clone());
        self
    }
    
    pub fn placeholder<T: IntoAttrib<UiState>>(mut self, attrib: T) -> Self {
        self.placeholder = attrib.into_attrib();
        self
    }
}

impl UiElementStub for TextBox {
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
