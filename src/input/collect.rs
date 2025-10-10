use crate::input::registry::ActionInputProcessor;
use crate::input::{Input, RawInputEvent};
use crate::window::Window;
use log::debug;
use mvutils::unsafe_utils::DangerousCell;
use parking_lot::Mutex;
use std::sync::Arc;

pub trait InputProcessor {
    fn digest_action(&mut self, action: RawInputEvent, input: &Input, window: &mut Window);
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
    targets: Vec<Arc<Mutex<dyn InputProcessor>>>,
    //ui: Arc<DangerousCell<Ui>>,
}

impl InputCollector {
    pub fn new(/*ui: Arc<DangerousCell<Ui>>*/) -> Self {
        Self {
            action_processor: ActionInputProcessor::new(),
            targets: vec![],
            //ui,
        }
    }

    pub fn register_new_event_target(&mut self, target: Arc<Mutex<dyn InputProcessor>>) {
        self.targets.push(target);
    }

    pub fn dispatch_input(&mut self, action: RawInputEvent, input: &Input, window: &mut Window) {
        #[cfg(feature = "timed")]
        {
            crate::debug::PROFILER.input(|t| t.resume());
        }
        //self.ui
        //    .get_mut()
        //    .digest_action(action.clone(), input, window);
        self.action_processor
            .digest_action(action.clone(), input, window);
        for target in &mut self.targets {
            let mut lock = target.lock();
            if lock.is_enabled() {
                lock.digest_action(action.clone(), input, window);
            }
        }
        #[cfg(feature = "timed")]
        {
            crate::debug::PROFILER.input(|t| t.pause());
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
