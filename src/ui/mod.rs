use crate::input::collect::InputProcessor;
use crate::input::{Input, RawInputEvent};
use crate::ui::context::{UiContext, UiResources};
use crate::ui::elements::{UiElement, UiElementCallbacks, UiElementStub};
use crate::ui::rendering::ctx::DrawContext2D;
use mvutils::once::{CreateOnce, Lazy};
use mvutils::unsafe_utils::{DangerousCell, Unsafe, UnsafeRef};
use mvutils::utils::Recover;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use crate::window::Window;

pub mod anim;
pub mod attributes;
pub mod context;
pub mod ease;
pub mod elements;
pub mod geometry;
pub mod parse;
pub mod prelude;
pub mod rendering;
pub mod res;
pub mod styles;
pub mod theme;
pub mod timing;
pub mod uix;
pub mod utils;

pub struct Ui {
    window: Option<&'static Window>, //Static is fine since Ui is member of Window and fucking rust lifetimes are so annoying to work with that im going the simple yet effective way :)
    context: CreateOnce<UiContext>,
    enabled: bool,
    root_elems: Vec<Rc<DangerousCell<UiElement>>>,
}

impl Ui {
    pub(crate) fn new() -> Self {
        Self {
            window: None,
            context: CreateOnce::new(),
            enabled: true,
            root_elems: vec![],
        }
    }
    
    pub fn init_window(&mut self, window: &Window) {
        self.window = Some(unsafe { Unsafe::cast_static(window) });
    }

    pub fn init(&mut self, resources: &'static dyn UiResources) {
        self.context.create(|| UiContext::new(resources));
    }

    pub fn context(&self) -> UiContext {
        self.context.deref().clone()
    }

    pub fn add_root(&mut self, elem: Rc<DangerousCell<UiElement>>) {
        self.root_elems.push(elem);
    }

    pub fn remove_root(&mut self, elem: Rc<DangerousCell<UiElement>>) {
        elem.get_mut().end_frame();
        self.root_elems.retain(|e| {
            let guard1 = e.get();
            let guard2 = elem.get();
            guard1.attributes().id != guard2.attributes().id
        });
    }

    pub fn compute_styles(&mut self, window: &Window) {
        for arc in self.root_elems.iter_mut() {
            let mut guard = arc.get_mut();
            guard.compute_styles(window);
        }
    }

    pub fn draw(&mut self, ctx: &mut DrawContext2D) {
        for arc in self.root_elems.iter_mut() {
            let mut guard = arc.get_mut();
            guard.draw(ctx);
        }
    }

    pub fn compute_styles_and_draw(&mut self, ctx: &mut DrawContext2D) {
        if let Some(window) = &self.window {
            for arc in self.root_elems.iter_mut() {
                let mut guard = arc.get_mut();
                guard.compute_styles(window);
                guard.draw(ctx);
            }
        }
    }
    
    pub fn end_frame(&mut self) {
        for arc in self.root_elems.iter_mut() {
            let mut guard = arc.get_mut();
            guard.end_frame();
        }
    }
}

impl InputProcessor for Ui {
    fn digest_action(&mut self, action: RawInputEvent, input: &Input, window: &mut Window) {
        for root in &self.root_elems {
            let e = root.get_mut();
            e.raw_input(action.clone(), input);
        }
        match action {
            RawInputEvent::Keyboard(action) => unsafe {
                for root in Unsafe::cast_static(&self.root_elems) {
                    let mut guard = root.get_mut();
                    let mut guard_ref = Unsafe::cast_mut_static(&mut guard);
                    let mut events = &mut guard.state_mut().events;
                    events.keyboard_change(action.clone(), &mut *guard_ref, &*input);
                }
            },
            RawInputEvent::Mouse(action) => unsafe {
                for root in Unsafe::cast_static(&self.root_elems) {
                    let mut guard = root.get_mut();
                    let mut guard_ref = Unsafe::cast_mut_static(&mut guard);
                    let mut events = &mut guard.state_mut().events;
                    events.mouse_change(action.clone(), &mut *guard_ref, &*input, window);
                }
            },
        }
    }

    fn end_frame(&mut self) {}

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}
