use crate::ui::elements::{UiElement, UiElementCallbacks, UiElementStub};
use mvutils::once::{CreateOnce, Lazy};
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use mvutils::utils::Recover;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use crate::input::collect::InputProcessor;
use crate::input::{Input, RawInputEvent};
use crate::ui::context::{UiContext, UiResources};
use crate::ui::rendering::ctx::DrawContext2D;

pub mod anim;
pub mod attributes;
pub mod ease;
pub mod elements;
pub mod parse;
pub mod prelude;
pub mod styles;
pub mod timing;
pub mod uix;
pub mod utils;
pub mod theme;
pub mod geometry;
pub mod rendering;
pub mod res;
pub mod context;

pub struct Ui {
    context: CreateOnce<UiContext>,
    enabled: bool,
    root_elems: Vec<Rc<DangerousCell<UiElement>>>,
}

impl Ui {
    pub(crate) fn new() -> Self {
        Self {
            context: CreateOnce::new(),
            enabled: true,
            root_elems: vec![],
        }
    }

    pub fn init(&mut self, resources: &'static dyn UiResources) {
        self.context.create(|| UiContext::new(resources));
    }

    pub fn context(&self) -> UiContext {
        self.context.clone()
    }

    pub fn add_root(&mut self, elem: Rc<DangerousCell<UiElement>>) {
        self.root_elems.push(elem);
    }

    pub fn remove_root(&mut self, elem: Rc<DangerousCell<UiElement>>) {
        self.root_elems.retain(|e| {
            let guard1 = e.get();
            let guard2 = elem.get();
            guard1.attributes().id != guard2.attributes().id
        })
    }

    pub fn compute_styles(&mut self) {
        for arc in self.root_elems.iter_mut() {
            let mut guard = arc.get_mut();
            guard.compute_styles();
        }
    }

    pub fn draw(&mut self, ctx: &mut DrawContext2D) {
        for arc in self.root_elems.iter_mut() {
            let mut guard = arc.get_mut();
            guard.draw(ctx);
        }
    }

    pub fn compute_styles_and_draw(&mut self, ctx: &mut DrawContext2D) {
        for arc in self.root_elems.iter_mut() {
            let mut guard = arc.get_mut();
            guard.compute_styles();
            guard.draw(ctx);
        }
    }
}

impl InputProcessor for Ui {
    fn digest_action(&mut self, action: RawInputEvent, input: &Input) {
        match action {
            RawInputEvent::Keyboard(action) => {
                unsafe {
                    for root in Unsafe::cast_static(&self.root_elems) {
                        let mut guard = root.get_mut();
                        let mut guard_ref = Unsafe::cast_mut_static(&mut guard);
                        let mut events = &mut guard.state_mut().events;
                        events.keyboard_change(action, &mut *guard_ref, &*input);
                    }
                }
            }
            RawInputEvent::Mouse(action) => {
                unsafe {
                    for root in Unsafe::cast_static(&self.root_elems) {
                        let mut guard = root.get_mut();
                        let mut guard_ref = Unsafe::cast_mut_static(&mut guard);
                        let mut events = &mut guard.state_mut().events;
                        events.mouse_change(action, &mut *guard_ref, &*input);
                    }
                }
            }
        }
    }

    fn end_frame(&mut self) {

    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}
