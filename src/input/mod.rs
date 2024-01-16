use crate::input::raw::{Input, State};
use crate::input::InputAction::{Keyboard, Mouse};
use mvutils::utils::Recover;
use std::rc::Rc;
use std::sync::RwLock;

pub mod raw;

pub(crate) struct InputCollector {
    default_processor: InputProcessorImpl,
    gui_processor: GuiInputProcessor,
    custom_processor: Option<Rc<RwLock<Box<dyn InputProcessor>>>>,
}

impl InputCollector {
    pub(crate) fn new(input: Rc<RwLock<Input>>) -> Self
    where
        Self: Sized,
    {
        Self {
            default_processor: InputProcessorImpl::new(input.clone()),
            gui_processor: GuiInputProcessor::new(input),
            custom_processor: None,
        }
    }

    pub(crate) fn get_input(&self) -> Rc<RwLock<Input>> {
        self.default_processor.input()
    }

    pub fn set_custom_processor(&mut self, custom_processor: Box<dyn InputProcessor>) {
        self.custom_processor = Some(Rc::new(RwLock::new(custom_processor)));
    }

    pub fn collect(&mut self, action: InputAction) {
        if let Keyboard(ka) = action {
            if self.default_processor.is_enabled() {
                self.default_processor.keyboard_change(ka);
            }
            if self.custom_processor.is_some() {
                let mut unwrapped = self.custom_processor.as_mut().unwrap().write().recover();
                if unwrapped.is_enabled() {
                    unwrapped.keyboard_change(ka);
                }
            }
        }
        if let Mouse(ma) = action {
            if self.default_processor.is_enabled() {
                self.default_processor.mouse_change(ma);
            }
            if self.custom_processor.is_some() {
                let mut unwrapped = self.custom_processor.as_mut().unwrap().write().recover();
                if unwrapped.is_enabled() {
                    unwrapped.mouse_change(ma)
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) enum InputAction {
    Keyboard(KeyboardAction),
    Mouse(MouseAction),
}

#[derive(Copy, Clone)]
pub enum KeyboardAction {
    Press(usize),
    Release(usize),
    Type(usize),
}

#[derive(Copy, Clone)]
pub enum MouseAction {
    Wheel(f32, f32),
    Move(i32, i32),
    Press(usize),
    Release(usize),
}

pub trait InputProcessor {
    fn new(input: Rc<RwLock<Input>>) -> Self
    where
        Self: Sized;
    fn input(&self) -> Rc<RwLock<Input>>;
    fn mouse_change(&mut self, action: MouseAction);
    fn keyboard_change(&mut self, action: KeyboardAction);
    fn set_enabled(&mut self, enabled: bool);
    fn enable(&mut self) {
        self.set_enabled(true);
    }
    fn disable(&mut self) {
        self.set_enabled(false);
    }
    fn toggle(&mut self) {
        self.set_enabled(!self.is_enabled());
    }
    fn is_enabled(&self) -> bool;
}

pub struct InputProcessorImpl {
    input: Rc<RwLock<Input>>,
    enabled: bool,
}

impl InputProcessor for InputProcessorImpl {
    fn new(input: Rc<RwLock<Input>>) -> Self {
        Self {
            input,
            enabled: true,
        }
    }

    fn input(&self) -> Rc<RwLock<Input>> {
        self.input.clone()
    }

    fn mouse_change(&mut self, action: MouseAction) {
        let mut input = self.input.write().recover();
        if let MouseAction::Press(btn) = action {
            input.mouse[btn] = true;
            input.mousestates[btn] = State::JustPressed;
        }

        if let MouseAction::Release(btn) = action {
            input.mousestates[btn] = State::JustReleased;
        }

        if let MouseAction::Wheel(x, y) = action {
            if y > 0.0 {
                input.scroll[raw::MOUSE_SCROLL_UP] = true;
                input.scrollstates[raw::MOUSE_SCROLL_UP] = y;
            }
            if y < 0.0 {
                input.scroll[raw::MOUSE_SCROLL_DOWN] = true;
                input.scrollstates[raw::MOUSE_SCROLL_DOWN] = y;
            }
            if x > 0.0 {
                input.scroll[raw::MOUSE_SCROLL_RIGHT] = true;
                input.scrollstates[raw::MOUSE_SCROLL_RIGHT] = x;
            }
            if x < 0.0 {
                input.scroll[raw::MOUSE_SCROLL_LEFT] = true;
                input.scrollstates[raw::MOUSE_SCROLL_LEFT] = x;
            }
        }

        if let MouseAction::Move(x, y) = action {
            input.positions[raw::MOUSE_POS_X] = x;
            input.positions[raw::MOUSE_POS_Y] = y;
        }
    }

    fn keyboard_change(&mut self, action: KeyboardAction) {
        let mut input = self.input.write().recover();
        if let KeyboardAction::Press(key) = action {
            input.keys[key] = true;
            input.keystates[key] = State::JustPressed;
        }
        if let KeyboardAction::Release(key) = action {
            input.keystates[key] = State::JustReleased;
        }
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

pub(crate) struct GuiInputProcessor {
    input: Rc<RwLock<Input>>,
    enabled: bool,
}

impl InputProcessor for GuiInputProcessor {
    fn new(input: Rc<RwLock<Input>>) -> Self
    where
        Self: Sized,
    {
        Self {
            input,
            enabled: true,
        }
    }

    fn input(&self) -> Rc<RwLock<Input>> {
        todo!()
    }

    fn mouse_change(&mut self, action: MouseAction) {
        todo!()
    }

    fn keyboard_change(&mut self, action: KeyboardAction) {
        todo!()
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}
