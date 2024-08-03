use std::ops::Deref;
use std::sync::{Arc, RwLock};
use mvutils::once::{CreateOnce, Lazy};
use mvutils::unsafe_utils::DangerousCell;
use mvutils::utils::Recover;
use crate::input::{InputProcessor, KeyboardAction, MouseAction};
use crate::input::raw::Input;
use crate::ui::elements::UiElement;

pub mod anim;
pub mod attributes;
pub mod background;
pub mod drawable;
pub mod ease;
pub mod elements;
pub mod parse;
pub mod parsing;
pub mod prelude;
pub mod styles;
pub mod timing;
pub mod utils;

pub(crate) static mut UI: Lazy<DangerousCell<Ui>> = Lazy::new(|| DangerousCell::new(Ui::new()));

pub struct Ui {
    input: CreateOnce<Arc<RwLock<Input>>>,
    enabled: bool,
    root_elems: Vec<Arc<parking_lot::RwLock<dyn UiElement>>>
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

    pub(crate) fn init_input(&mut self, input: Arc<RwLock<Input>>) {
        self.input.create(move || input);
    }

    pub fn add_root(&mut self, elem: Arc<parking_lot::RwLock<dyn UiElement>>) {
        self.root_elems.push(elem);
    }

    pub fn remove_root(&mut self, elem: Arc<parking_lot::RwLock<dyn UiElement>>) {
        self.root_elems.retain(|e| {
            let guard1 = e.read();
            let guard2 = elem.read();
            guard1.attributes().id != guard2.attributes().id
        })
    }
}

impl InputProcessor for Ui {
    fn new(input: Arc<RwLock<Input>>) -> Self where Self: Sized {
        unimplemented!()
    }

    fn input(&self) -> Arc<RwLock<Input>> {
        self.input.clone()
    }

    fn mouse_change(&mut self, action: MouseAction) {
        let input = self.input.read().recover();

        //for root in self.root_elems {
        //    let mut guard = root.write();
        //    //let mut events = &guard.state_mut().events;
        //    //events.mouse_change(action, &guard, &*input);
        //}
    }

    fn keyboard_change(&mut self, action: KeyboardAction) {
        let input = self.input.read().recover();

        //for root in self.root_elems {
        //    let mut guard = root.write();
        //    //let mut events = &guard.state_mut().events;
        //    //events.keyboard_change(action, &guard, &*input);
        //}
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}