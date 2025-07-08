pub mod transition;

use log::warn;
use crate::input::{Input, KeyboardAction, MouseAction, RawInputEvent};
use crate::rendering::OpenGLRenderer;
use crate::rendering::pipeline::RenderingPipeline;
use crate::ui::elements::{Element, UiElementCallbacks, UiElementStub};
use crate::ui::geometry::SimpleRect;
use crate::window::Window;

pub struct UiPageManager {
    available: Vec<Element>,
    current: Option<Element>,
    history: Vec<Element>
}

impl UiPageManager {
    pub fn new() -> Self {
        Self {
            available: vec![],
            current: None,
            history: vec![],
        }
    }

    pub fn add_page(&mut self, page: impl Page) {
        let e = page.get_elem();
        if self.current.is_none() {
            self.current = Some(e.clone());
        }
        self.available.push(e);
    }

    fn find_page(&self, name: &str) -> Option<Element> {
        self.available.iter()
            .find(|e| e.get().attributes().id.as_deref() == Some(name))
            .cloned()
    }

    //TODO: make transitions lmao
    pub fn transition(&mut self, target: &str) {
        if let Some(target_elem) = self.find_page(target) {
            //current will be some as find_page wouldnt return anything otherwise, still go safe route lmao
            if let Some(current) = self.current.take() {
                self.history.push(current);
                target_elem.get_mut().state_mut().invalidate();
                self.current = Some(target_elem);
            }
        } else {
            warn!("Cannot fing page: {}", target);
        }
    }

    pub fn close_all(&mut self) {
        self.current = None;
    }

    pub fn open(&mut self, target: &str) {
        if let Some(target_elem) = self.find_page(target) {
            target_elem.get_mut().state_mut().invalidate();
            if let Some(current) = self.current.take() {
                self.history.push(current);
            }
            self.current = Some(target_elem);
        } else {
            warn!("Cannot fing page: {}", target);
        }
    }

    pub fn go_back(&mut self) -> Option<String> {
        if let Some(new) = self.history.pop() {
            new.get_mut().state_mut().invalidate();
            //I dont think pushing to the history would be the expected behavior
            let id = new.get().attributes().id.clone();
            self.current = Some(new);
            id
        } else {
            None
        }
    }

    pub fn draw(&mut self, ctx: &mut RenderingPipeline<OpenGLRenderer>, crop_area: &SimpleRect) {
        if let Some(current) = &mut self.current {
            current.get_mut().frame_callback(ctx, crop_area);
        }
    }

    pub fn raw_input(&mut self, raw_input_event: RawInputEvent, input: &Input) {
        if let Some(current) = &mut self.current {
            current.get_mut().raw_input(raw_input_event, input);
        }
    }

    pub fn keyboard_change(&mut self, action: KeyboardAction, input: &Input, window: &mut Window) {
        if let Some(current) = &mut self.current {
            current.get_mut().state_mut().events.keyboard_change(action, current.get_mut(), input);
        }
    }

    pub fn mouse_change(&mut self, action: MouseAction, input: &Input, window: &mut Window) {
        if let Some(current) = &mut self.current {
            current.get_mut().state_mut().events.mouse_change(action, current.get_mut(), input, window);
        }
    }

    pub fn end_frame(&mut self) {
        if let Some(current) = &mut self.current {
            current.get_mut().end_frame();
        }
    }

    pub fn invalidate(&mut self) {
        if let Some(current) = &mut self.current {
            current.get_mut().state_mut().invalidate();
        }
    }
}

pub trait Page {
    fn get_elem(&self) -> Element;
}

impl Page for Element {
    fn get_elem(&self) -> Element {
        self.clone()
    }
}