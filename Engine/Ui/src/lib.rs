use std::ops::Deref;
use std::sync::{Arc, RwLock};
use mvutils::once::{CreateOnce, Lazy};
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use mvutils::utils::Recover;
use mvcore::input::{InputAction, InputProcessor, KeyboardAction, MouseAction};
use mvcore::input::raw::Input;
use crate::elements::{UiElement, UiElementStub};

pub mod anim;
pub mod attributes;
pub mod drawable;
pub mod ease;
pub mod elements;
pub mod parse;
pub mod parsing;
pub mod prelude;
pub mod styles;
pub mod timing;
pub mod utils;
mod shapes;

pub static mut UI: Lazy<Arc<DangerousCell<Ui>>> = Lazy::new(|| Arc::new(DangerousCell::new(Ui::new())));

pub struct Ui {
    input: CreateOnce<Arc<DangerousCell<Input>>>,
    enabled: bool,
    root_elems: Vec<Arc<parking_lot::RwLock<UiElement>>>
}

impl Ui {
    fn new() -> Self {
        unsafe {
            if UI.created() {
            }
        }

        Self {
            input: CreateOnce::new(),
            enabled: true,
            root_elems: vec![],
        }
    }

    pub fn init_input(&mut self, input: Arc<DangerousCell<Input>>) {
        self.input.create(move || input);
    }

    pub fn add_root(&mut self, elem: Arc<parking_lot::RwLock<UiElement>>) {
        self.root_elems.push(elem);
    }

    pub fn remove_root(&mut self, elem: Arc<parking_lot::RwLock<UiElement>>) {
        self.root_elems.retain(|e| {
            let guard1 = e.read();
            let guard2 = elem.read();
            guard1.attributes().id != guard2.attributes().id
        })
    }

    pub fn input_processor(action: InputAction) {
        match action {
            InputAction::Keyboard(k) => unsafe {
                UI.get_mut().keyboard_change(k);
            }
            InputAction::Mouse(m) => unsafe {
                UI.get_mut().mouse_change(m);
            }
        }
    }
}

impl InputProcessor for Ui {
    fn new(input: Arc<DangerousCell<Input>>) -> Self where Self: Sized {
        unimplemented!()
    }

    fn input(&self) -> Arc<DangerousCell<Input>> {
        self.input.clone()
    }

    fn mouse_change(&mut self, action: MouseAction) {
        let input = self.input.get_mut();

        unsafe {
            for root in Unsafe::cast_static(&self.root_elems) {
                let mut guard = root.write();
                let mut guard_ref = Unsafe::cast_mut_static(&mut guard);
                let mut events = &mut guard.state_mut().events;
                events.mouse_change(action, &mut *guard_ref, &*input);
            }
        }
    }

    fn keyboard_change(&mut self, action: KeyboardAction) {
        let input = self.input.get_mut();


        unsafe {
            for root in Unsafe::cast_static(&self.root_elems) {
                let mut guard = root.write();
                let mut guard_ref = Unsafe::cast_mut_static(&mut guard);
                let mut events = &mut guard.state_mut().events;
                events.keyboard_change(action, &mut *guard_ref, &*input);
            }
        }
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}