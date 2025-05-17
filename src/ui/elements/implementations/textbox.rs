use crate::enum_val_ref_mut;
use crate::input::consts::{Key, MouseButton};
use crate::input::registry::RawInput;
use crate::input::{Input, KeyboardAction, MouseAction, RawInputEvent};
use crate::ui::attributes::{Attributes, UiState};
use crate::ui::context::UiContext;
use crate::ui::elements::child::Child;
use crate::ui::elements::components::edittext::EditableTextHelper;
use crate::ui::elements::components::text::TextBody;
use crate::ui::elements::components::ElementBody;
use crate::ui::elements::{Element, UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use crate::ui::rendering::ctx::DrawContext2D;
use crate::ui::styles::UiStyle;
use mvutils::state::State;
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use std::rc::{Rc, Weak};
use crate::ui::styles::types::Dimension;

#[derive(Clone)]
pub struct TextBox {
    rc: Weak<DangerousCell<UiElement>>,
    
    context: UiContext,
    state: UiElementState,
    style: UiStyle,
    attributes: Attributes,
    body: ElementBody,
    text_body: TextBody<TextBox>,
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
}

impl UiElementCallbacks for TextBox {
    fn draw(&mut self, ctx: &mut DrawContext2D) {
        let this = unsafe { Unsafe::cast_static(self) };
        self.body.draw(this, ctx, &self.context);
        for children in &self.state.children {
            match children {
                Child::Element(e) => {
                    let guard = e.get_mut();
                    guard.draw(ctx);
                }
                _ => {}
            }
        }
        let s = self.content.read();
        if s.is_empty() {
            if !self.focused {
                let placeholder = self.placeholder.read();
                self.text_body.draw(placeholder.as_str(), this, ctx, &self.context);
            } else {
                self.helper.draw(this, ctx, &self.context);
            }
        } else {
            if self.focused {
                self.helper.draw(this, ctx, &self.context);
            }
            self.text_body.draw(&s[self.helper.view_range.clone()], this, ctx, &self.context);
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
                            let is_shift = input.action_processor().is_raw_input(RawInput::KeyPress(Key::LShift));
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
            RawInputEvent::Mouse(ma) => {
                match ma {
                    MouseAction::Press(p) => {
                        let mx = input.mouse_x;
                        let my = input.mouse_y;
                        if let MouseButton::Left = p {
                            if self.inside(mx, my) {
                                self.focused = true;
                            } else {
                                self.focused = false;
                            }
                        }
                    }
                    _ => {}
                }
            }
        };

        true
    }
}

impl UiElementStub for TextBox {
    fn new(context: UiContext, attributes: Attributes, style: UiStyle) -> Element
    where
        Self: Sized
    {
        let content = match attributes.attribs.get("content") {
            None => State::new(String::new()).map_identity(),
            Some(v) => v.as_ui_state()
        };

        let placeholder = match attributes.attribs.get("placeholder") {
            None => State::new(String::new()).map_identity(),
            Some(v) => v.as_ui_state()
        };
        
        let this = Self {
            rc: Weak::new(),
            context,
            state: UiElementState::new(),
            style,
            attributes,
            body: ElementBody::new(),
            text_body: TextBody::new(),
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

    fn get_size(&self, s: &str) -> Dimension<i32> {
        todo!()
    }
}