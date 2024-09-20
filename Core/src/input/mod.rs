use crate::input::raw::Input;
use crate::input::InputAction::{Keyboard, Mouse};
pub use consts::*;
use mvutils::unsafe_utils::{DangerousCell, Nullable, Unsafe};
use mvutils::utils::Recover;
pub use raw::State;
use std::sync::{Arc, RwLock};

mod consts;
pub mod raw;

pub struct InputCollector {
    default_processor: InputProcessorImpl,
    custom_processor: Option<fn(InputAction)>,
}

impl InputCollector {
    pub(crate) fn new(input: Arc<DangerousCell<Input>>) -> Self
    where
        Self: Sized,
    {
        Self {
            default_processor: InputProcessorImpl::new(input.clone()),
            custom_processor: None,
        }
    }

    pub(crate) fn get_input(&self) -> Arc<DangerousCell<Input>> {
        self.default_processor.input()
    }

    pub fn set_custom_processor(&mut self, custom_processor: fn(InputAction)) {
        unsafe {
            self.custom_processor = Some(custom_processor);
        }
    }

    pub(crate) fn collect(&mut self, action: InputAction) {
        if let Keyboard(ka) = action {
            if self.default_processor.is_enabled() {
                self.default_processor.keyboard_change(ka);
            }
        }
        if let Mouse(ma) = action {
            if self.default_processor.is_enabled() {
                self.default_processor.mouse_change(ma);
            }
        }
        if self.custom_processor.is_some() {
            self.custom_processor.unwrap()(action);
        }
    }
}

#[derive(Copy, Clone)]
pub enum InputAction {
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
    fn new(input: Arc<DangerousCell<Input>>) -> Self
    where
        Self: Sized;
    fn input(&self) -> Arc<DangerousCell<Input>>;
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
    input: Arc<DangerousCell<Input>>,
    enabled: bool,
}

impl InputProcessor for InputProcessorImpl {
    fn new(input: Arc<DangerousCell<Input>>) -> Self {
        Self {
            input,
            enabled: true,
        }
    }

    fn input(&self) -> Arc<DangerousCell<Input>> {
        self.input.clone()
    }

    fn mouse_change(&mut self, action: MouseAction) {
        let mut input = self.input.get_mut();
        if let MouseAction::Press(btn) = action {
            input.mouse[btn] = true;
            input.mousestates[btn] = State::JustPressed;
        }

        if let MouseAction::Release(btn) = action {
            input.mousestates[btn] = State::JustReleased;
        }

        if let MouseAction::Wheel(x, y) = action {
            if y > 0.0 {
                input.scroll[MOUSE_SCROLL_UP] = true;
                input.scrollstates[MOUSE_SCROLL_UP] = y;
            }
            if y < 0.0 {
                input.scroll[MOUSE_SCROLL_DOWN] = true;
                input.scrollstates[MOUSE_SCROLL_DOWN] = y;
            }
            if x > 0.0 {
                input.scroll[MOUSE_SCROLL_RIGHT] = true;
                input.scrollstates[MOUSE_SCROLL_RIGHT] = x;
            }
            if x < 0.0 {
                input.scroll[MOUSE_SCROLL_LEFT] = true;
                input.scrollstates[MOUSE_SCROLL_LEFT] = x;
            }
        }

        if let MouseAction::Move(x, y) = action {
            input.positions[MOUSE_POS_X] = x;
            input.positions[MOUSE_POS_Y] = y;
        }
    }

    fn keyboard_change(&mut self, action: KeyboardAction) {
        let mut input = self.input.get_mut();
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
