use crate::input::consts::{Key, MouseButton};
use crate::input::registry::RawInput;
use crate::input::{Input, KeyboardAction, MouseAction, RawInputEvent};
use crate::rendering::{OpenGLRenderer, RenderContext};
use crate::ui::attributes::{Attributes, UiState};
use crate::ui::context::UiContext;
use crate::ui::elements::child::Child;
use crate::ui::elements::components::ElementBody;
use crate::ui::elements::components::boring::BoringText;
use crate::ui::elements::components::edittext::EditableTextHelper;
use crate::ui::elements::{create_style_obs, Element, UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use crate::ui::geometry::SimpleRect;
use crate::ui::rendering::UiRenderer;
use crate::ui::styles::{UiStyle, UiStyleWriteObserver};
use crate::ui::styles::types::Dimension;
use mvutils::enum_val_ref_mut;
use mvutils::state::State;
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use std::rc::{Rc, Weak};
use crate::rendering::pipeline::RenderingPipeline;

#[derive(Clone)]
pub struct TextBox {
    rc: Weak<DangerousCell<UiElement>>,

    context: UiContext,
    state: UiElementState,
    style: UiStyle,
    attributes: Attributes,
    body: ElementBody,
    content: UiState,
    placeholder: UiState,
    focused: bool,
    helper: EditableTextHelper<TextBox>,
}

impl TextBox {
    pub fn content(&self) -> UiState {
        self.content.clone()
    }

    pub fn placeholder(&self) -> UiState {
        self.placeholder.clone()
    }

    pub fn focus_now(&mut self) {
        self.focused = true;
    }
}

impl UiElementCallbacks for TextBox {
    fn draw(&mut self, ctx: &mut RenderingPipeline<OpenGLRenderer>, crop_area: &SimpleRect) {
        let this = unsafe { Unsafe::cast_static(self) };
        self.body.draw(this, ctx, &self.context, crop_area);
        for children in &self.state.children {
            match children {
                Child::Element(e) => {
                    let guard = e.get_mut();
                    guard.frame_callback(ctx, crop_area);
                }
                _ => {}
            }
        }
        let s = self.content.read();
        if s.is_empty() {
            if !self.focused {
                let placeholder = self.placeholder.read();
                self.helper.draw_other(&*placeholder, this, ctx, &self.context, crop_area);
            } else {
                self.helper.draw(this, ctx, &self.context, crop_area, true);
            }
        } else {
            if self.focused {
                self.helper.draw(this, ctx, &self.context, crop_area, true);
            } else {
                self.helper.draw(this, ctx, &self.context, crop_area, false);
            }
        }
    }

    fn raw_input(&mut self, action: RawInputEvent, input: &Input) -> bool {
        let unsafe_self = unsafe { Unsafe::cast_mut_static(self) };
        self.body.on_input(unsafe_self, action.clone(), input);

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
                            self.helper.add_str(&ch.to_string());
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

impl UiElementStub for TextBox {
    fn new(context: UiContext, attributes: Attributes, style: UiStyle) -> Element
    where
        Self: Sized,
    {
        let content = match attributes.attribs.get("content") {
            None => State::new(String::new()).map_identity(),
            Some(v) => v.as_ui_state(),
        };

        let placeholder = match attributes.attribs.get("placeholder") {
            None => State::new(String::new()).map_identity(),
            Some(v) => v.as_ui_state(),
        };

        let this = Self {
            rc: Weak::new(),
            context: context.clone(),
            state: UiElementState::new(context),
            style,
            attributes,
            body: ElementBody::new(),
            content: content.clone(),
            placeholder,
            focused: false,
            helper: EditableTextHelper::new(content),
        };

        let rc = Rc::new(DangerousCell::new(this.wrap()));
        let e = rc.get_mut();
        let bx = enum_val_ref_mut!(UiElement, e, TextBox);
        bx.rc = Rc::downgrade(&rc);

        rc
    }

    fn wrap(self) -> UiElement {
        UiElement::TextBox(self)
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
