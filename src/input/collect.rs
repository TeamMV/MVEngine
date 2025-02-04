use std::sync::Arc;
use log::__private_api::loc;
use parking_lot::Mutex;
use crate::input::{Input, RawInputEvent};
use crate::input::registry::ActionInputProcessor;

pub trait InputProcessor {
    fn digest_action(&mut self, action: RawInputEvent, input: &Input);
    fn end_frame(&mut self);
    fn set_enabled(&mut self, state: bool);
    fn is_enabled(&self) -> bool;

    fn enable(&mut self) {
        self.set_enabled(true);
    }

    fn disable(&mut self) {
        self.set_enabled(false);
    }
}

pub struct InputCollector {
    pub(crate) action_processor: ActionInputProcessor,
    targets: Vec<Arc<Mutex<dyn InputProcessor>>>
}

impl InputCollector {
    pub fn new() -> Self {
        Self {
            action_processor: ActionInputProcessor::new(),
            targets: vec![],
        }
    }

    pub fn register_new_event_target(&mut self, target: Arc<Mutex<dyn InputProcessor>>) {
        self.targets.push(target);
    }

    pub fn dispatch_input(&mut self, action: RawInputEvent, input: &Input) {
        self.action_processor.digest_action(action, input);
        for target in &mut self.targets {
            let mut lock = target.lock();
            if lock.is_enabled() {
                lock.digest_action(action, input);
            }
        }
    }

    pub fn end_frame(&mut self) {
        self.action_processor.end_frame();
        for target in &mut self.targets {
            let mut lock = target.lock();
            if lock.is_enabled() {
                lock.end_frame();
            }
        }
    }
}