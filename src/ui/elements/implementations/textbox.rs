use crate::input::consts::{Key, MouseButton};
use crate::input::{Input, KeyboardAction, MouseAction, RawInputEvent};
use crate::ui::attributes::{AttributeValue, Attributes};
use crate::ui::context::UiContext;
use crate::ui::elements::child::Child;
use crate::ui::elements::components::ElementBody;
use crate::ui::elements::{UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use crate::ui::rendering::ctx::DrawContext2D;
use crate::ui::styles::{Dimension, UiStyle};
use mvutils::state::State;
use mvutils::unsafe_utils::Unsafe;
use crate::input::registry::RawInput;
use crate::ui::elements::button::Button;
use crate::ui::elements::components::edittext::EditableTextHelper;
use crate::ui::elements::components::text::TextBody;

#[derive(Clone)]
pub struct TextBox {
    context: UiContext,
    state: UiElementState,
    style: UiStyle,
    attributes: Attributes,
    body: ElementBody<TextBox>,
    text_body: TextBody<TextBox>,
    content: State<String>,
    placeholder: State<String>,
    focused: bool,
    helper: EditableTextHelper<TextBox>,
}

impl TextBox {
    pub fn body(&self) -> &ElementBody<TextBox> {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut ElementBody<TextBox> {
        &mut self.body
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
    fn new(context: UiContext, attributes: Attributes, style: UiStyle) -> Self
    where
        Self: Sized
    {
        let content = match attributes.attribs.get("content") {
            None => String::new(),
            Some(v) => {
                match v {
                    AttributeValue::Str(s) => s.clone(),
                    AttributeValue::Int(i) => i.to_string(),
                    AttributeValue::Float(f) => f.to_string(),
                    AttributeValue::Bool(b) => b.to_string(),
                    AttributeValue::Char(c) => c.to_string(),
                    _ => "fn(Element)".to_string()
                }
            }
        };

        let placeholder = match attributes.attribs.get("placeholder") {
            None => String::new(),
            Some(v) => {
                match v {
                    AttributeValue::Str(s) => s.clone(),
                    AttributeValue::Int(i) => i.to_string(),
                    AttributeValue::Float(f) => f.to_string(),
                    AttributeValue::Bool(b) => b.to_string(),
                    AttributeValue::Char(c) => c.to_string(),
                    _ => "fn(Element)".to_string()
                }
            }
        };

        let content = State::new(content);
        
        Self {
            context,
            state: UiElementState::new(),
            style,
            attributes,
            body: ElementBody::new(),
            text_body: TextBody::new(),
            content: content.clone(),
            placeholder: State::new(placeholder),
            focused: false,
            helper: EditableTextHelper::new(content),
        }
    }

    fn wrap(self) -> UiElement {
        UiElement::TextBox(self)
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