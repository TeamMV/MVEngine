use crate::input::collect::InputProcessor;
use crate::input::{Input, RawInputEvent};
use crate::rendering::pipeline::RenderingPipeline;
use crate::rendering::{OpenGLRenderer, RenderContext};
use crate::ui::context::{UiContext, UiResources};
use crate::ui::elements::{UiElement, UiElementCallbacks, UiElementStub};
use crate::ui::geometry::SimpleRect;
use crate::ui::rendering::WideRenderContext;
use crate::window::Window;
use mvutils::once::CreateOnce;
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use std::ops::Deref;
use std::rc::Rc;
use crate::ui::page::UiPageManager;

pub mod anim;
pub mod attributes;
pub mod context;
pub mod ease;
pub mod elements;
pub mod geometry;
pub mod mss;
pub mod parse;
pub mod prelude;
pub mod rendering;
pub mod res;
pub mod styles;
pub mod uix;
pub mod utils;
pub mod page;

pub struct Ui {
    context: CreateOnce<UiContext>,
    enabled: bool,
    root_elems: Vec<Rc<DangerousCell<UiElement>>>,
    page_manager: UiPageManager
}

impl Ui {
    pub(crate) fn new() -> Self {
        Self {
            context: CreateOnce::new(),
            enabled: true,
            root_elems: vec![],
            page_manager: UiPageManager::new(),
        }
    }

    pub fn init(&mut self, resources: &'static dyn UiResources) {
        self.context.create(|| UiContext::new(resources));
    }

    pub fn context(&self) -> UiContext {
        self.context.deref().clone()
    }

    pub fn add_root(&mut self, elem: Rc<DangerousCell<UiElement>>) {
        elem.get_mut().state_mut().invalidate();
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

    pub fn invalidate(&mut self) {
        for arc in self.root_elems.iter_mut() {
            arc.get_mut().state_mut().invalidate();
        }
        self.page_manager.invalidate();
    }

    pub fn compute_styles(&mut self, ctx: &mut impl WideRenderContext) {
        for arc in self.root_elems.iter_mut() {
            let guard = arc.get_mut();
            guard.compute_styles(ctx);
        }
    }

    pub fn draw(&mut self, ctx: &mut RenderingPipeline<OpenGLRenderer>, crop_area: &SimpleRect) {
        for arc in self.root_elems.iter_mut() {
            let guard = arc.get_mut();
            guard.frame_callback(ctx, crop_area);
        }
        self.page_manager.draw(ctx, crop_area);
    }

    pub fn end_frame(&mut self) {
        for arc in self.root_elems.iter_mut() {
            let guard = arc.get_mut();
            guard.end_frame();
        }
        self.page_manager.end_frame();
    }

    pub fn page_manager(&self) -> &UiPageManager {
        &self.page_manager
    }

    pub fn page_manager_mut(&mut self) -> &mut UiPageManager {
        &mut self.page_manager
    }
}

impl InputProcessor for Ui {
    fn digest_action(&mut self, action: RawInputEvent, input: &Input, window: &mut Window) {
        for root in &self.root_elems {
            let e = root.get_mut();
            e.raw_input(action.clone(), input);
        }
        self.page_manager.raw_input(action.clone(), input);
        match action {
            RawInputEvent::Keyboard(action) => unsafe {
                for root in Unsafe::cast_lifetime(&self.root_elems) {
                    let mut guard = root.get_mut();
                    let guard_ref = Unsafe::cast_lifetime_mut(&mut guard);
                    let events = &mut guard.state_mut().events;
                    events.keyboard_change(action.clone(), &mut *guard_ref, &*input);
                }
                self.page_manager.keyboard_change(action, input, window);
            },
            RawInputEvent::Mouse(action) => unsafe {
                for root in Unsafe::cast_lifetime(&self.root_elems) {
                    let mut guard = root.get_mut();
                    let guard_ref = Unsafe::cast_lifetime_mut(&mut guard);
                    let events = &mut guard.state_mut().events;
                    events.mouse_change(action.clone(), &mut *guard_ref, &*input, window);
                }
                self.page_manager.mouse_change(action, input, window);
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
