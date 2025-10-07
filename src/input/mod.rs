use crate::input::collect::{InputCollector, InputProcessor};
use crate::input::consts::{Key, MouseButton};
use crate::input::registry::{ActionInputProcessor, InputRegistry};
//use crate::ui::Ui;
use bitflags::Bits;
use log::error;
use mvutils::unsafe_utils::DangerousCell;
use parking_lot::Mutex;
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::sync::Arc;

pub mod collect;
pub mod consts;
pub mod registry;

#[derive(Clone, Debug)]
pub enum RawInputEvent {
    Keyboard(KeyboardAction),
    Mouse(MouseAction),
}

#[derive(Clone, Debug)]
pub enum KeyboardAction {
    Press(Key),
    Release(Key),
    Type(Key),
    Char(char),
}

#[derive(Clone, Debug)]
pub enum MouseAction {
    Wheel(f32, f32),
    Move(i32, i32),
    Press(MouseButton),
    Release(MouseButton),
}

pub const FILENAME: &str = "actions.sav";

pub struct Input {
    pub(crate) collector: InputCollector,
    pub mouse_x: i32,
    pub mouse_y: i32,
}

impl Input {
    pub fn new(/*ui: Arc<DangerousCell<Ui>>*/) -> Self {
        Self {
            collector: InputCollector::new(/*ui*/),
            mouse_x: i32::EMPTY,
            mouse_y: i32::EMPTY,
        }
    }

    pub fn register_new_event_target(&mut self, target: Arc<Mutex<dyn InputProcessor>>) {
        self.collector.register_new_event_target(target);
    }

    pub fn is_action(&self, name: &str) -> bool {
        self.collector
            .action_processor
            .registry
            .is_action_triggered(name)
    }

    pub fn was_action(&self, name: &str) -> bool {
        self.collector
            .action_processor
            .registry
            .was_action_triggered(name)
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

    pub fn save_actions(&self, to: &PathBuf) -> std::io::Result<()> {
        let mut to = to.clone();
        to.push(FILENAME);
        let mut file = File::options().write(true).open(&to);
        if let Err(_) = file {
            file = File::create(&to);
        }
        if let Ok(mut file) = file {
            let reg = &self.collector.action_processor.registry;
            reg.save_to_file(&mut file)
        } else {
            error!("Could not create or read actions file");
            Err(Error::from(ErrorKind::NotFound))
        }
    }

    pub fn load_actions(&mut self, from: &PathBuf) -> Result<(), String> {
        let mut from = from.clone();
        from.push(FILENAME);
        let mut file = File::options()
            .read(true)
            .open(&from)
            .map_err(|x| x.to_string())?;

        let reg = &mut self.collector.action_processor.registry;
        reg.load_from_file(&mut file)
    }
}
