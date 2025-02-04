use std::sync::Arc;
use bitflags::Bits;
use parking_lot::Mutex;
use crate::input::collect::{InputCollector, InputProcessor};
use crate::input::consts::{Key, MouseButton};
use crate::input::registry::{ActionInputProcessor, InputRegistry};

pub mod collect;
pub mod registry;
pub mod consts;

#[derive(Copy, Clone)]
pub enum RawInputEvent {
    Keyboard(KeyboardAction),
    Mouse(MouseAction),
}

#[derive(Copy, Clone)]
pub enum KeyboardAction {
    Press(Key),
    Release(Key),
    Type(Key),
}

#[derive(Copy, Clone)]
pub enum MouseAction {
    Wheel(f32, f32),
    Move(i32, i32),
    Press(MouseButton),
    Release(MouseButton),
}

pub struct Input {
    pub(crate) collector: InputCollector,
    pub mouse_x: i32,
    pub mouse_y: i32,
}

impl Input {
    pub fn new() -> Self {
        Self {
            collector: InputCollector::new(),
            mouse_x: i32::EMPTY,
            mouse_y: i32::EMPTY,
        }
    }

    pub fn register_new_event_target(&mut self, target: Arc<Mutex<dyn InputProcessor>>) {
        self.collector.register_new_event_target(target);
    }

    pub fn is_action(&self, name: &str) -> bool {
        self.collector.action_processor.registry.is_action_triggered(name)
    }

    pub fn was_action(&self, name: &str) -> bool {
        self.collector.action_processor.registry.was_action_triggered(name)
    }

    pub fn action_processor(&self) -> &ActionInputProcessor {
        &self.collector.action_processor
    }

    pub fn action_processor_mut(&mut self) -> &mut ActionInputProcessor {
        &mut self.collector.action_processor
    }

    pub fn action_registry(&self) -> &InputRegistry {
        &self.collector.action_processor.registry
    }

    pub fn action_registry_mut(&mut self) -> &mut InputRegistry {
        &mut self.collector.action_processor.registry
    }
}

