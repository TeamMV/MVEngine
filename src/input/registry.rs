use std::collections::btree_map::Entry;
use std::fmt::Debug;
use std::hash::Hash;
use hashbrown::{HashSet, HashMap};
use crate::input::collect::InputProcessor;
use crate::input::{Input, KeyboardAction, MouseAction, RawInputEvent};
use crate::input::consts::{Key, MouseButton};

#[derive(Debug)]
pub enum State {
    Was,
    Is
}

pub struct InputRegistry {
    actions: Vec<(String, Vec<RawInput>)>,
    current: HashMap<String, State>
}

fn unordered_contains_all<T: Hash + Eq + Debug>(a: &HashSet<T>, b: &[T]) -> bool {
    for t in b {
        if !a.contains(t) {
            return false;
        }
    }
    true
}

impl InputRegistry {
    pub fn new() -> Self {
        Self {
            actions: vec![],
            current: HashMap::new(),
        }
    }

    pub fn create_action(&mut self, name: &str) {
        self.actions.push((name.to_string(), Vec::new()));
    }

    pub fn bind_action(&mut self, name: &str, components: Vec<RawInput>) {
        if let Some((_, some)) = self.actions.iter_mut().find(|(n, _)| n == name) {
            *some = components;
        }
    }

    pub fn process(&mut self, inputs: &HashSet<RawInput>) {
        for (action, required_inputs) in &self.actions {
            if unordered_contains_all(&inputs, required_inputs) {
                let _ = self.current.try_insert(action.clone(), State::Was);
            } else {
                self.current.remove(action);
            }
        }
    }

    pub fn end_frame(&mut self) {
        for (_, state) in self.current.iter_mut() {
            *state = State::Is;
        }
    }

    pub fn is_action_triggered(&self, what: &str) -> bool {
        self.current.contains_key(what)
    }

    pub fn was_action_triggered(&self, what: &str) -> bool {
        if let Some(some) = self.current.get(what) {
            matches!(some, State::Was)
        } else {
            false
        }
    }
}

pub struct ActionInputProcessor {
    enabled: bool,
    pub(crate) registry: InputRegistry,
    inputs: HashSet<RawInput>
}

impl ActionInputProcessor {
    pub(crate) fn new() -> Self {
        Self {
            enabled: true,
            registry: InputRegistry::new(),
            inputs: Default::default(),
        }
    }
}

impl InputProcessor for ActionInputProcessor {
    fn digest_action(&mut self, action: RawInputEvent, input: &Input) {
        let raw_input = RawInput::from_raw(action, input);
        if let Some(raw_input) = raw_input {
            let _ = match raw_input {
                RawInput::MouseRelease(button) => self.inputs.remove(&RawInput::MousePress(button)),
                RawInput::KeyRelease(key) => self.inputs.remove(&RawInput::KeyPress(key)),
                _ => self.inputs.insert(raw_input),
            };
        }
    }

    fn end_frame(&mut self) {
        self.registry.end_frame();
        self.registry.process(&self.inputs);
        //self.inputs.retain(|input| matches!(input, RawInput::MouseRelease(_) | RawInput::KeyRelease(_)))
        //bro i genuiely have no idea what you were trying to achieve here
    }

    fn set_enabled(&mut self, state: bool) {
        self.enabled = state;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum RawInput {
    MousePress(MouseButton),
    MouseRelease(MouseButton),
    MouseMove(Direction),
    KeyPress(Key),
    KeyRelease(Key),
    Scroll(Direction),
}

impl RawInput {
    pub fn from_raw(event: RawInputEvent, input: &Input) -> Option<Self> {
        match event {
            RawInputEvent::Keyboard(keyboard_event) => {
                match keyboard_event {
                    KeyboardAction::Press(press_event) => {
                        Some(RawInput::KeyPress(press_event))
                    }
                    KeyboardAction::Release(release_event) => {
                        Some(RawInput::KeyRelease(release_event))
                    }
                    KeyboardAction::Type(type_event) => {
                        None
                    }
                }
            }
            RawInputEvent::Mouse(mouse_event) => {
                match mouse_event {
                    MouseAction::Wheel(dx, dy) => {
                        if dx.abs() > dy.abs() {
                            if dx > 0.0 {
                                Some(RawInput::Scroll(Direction::Right))
                            } else {
                                Some(RawInput::Scroll(Direction::Left))
                            }
                        } else {
                            if dy > 0.0 {
                                Some(RawInput::Scroll(Direction::Up))
                            } else {
                                Some(RawInput::Scroll(Direction::Down))
                            }
                        }
                    }
                    MouseAction::Move(to_x, to_y) => {
                        let dx = input.mouse_x - to_x;
                        let dy = input.mouse_y - to_y;

                        if dx.abs() > dy.abs() {
                            if dx > 0 {
                                Some(RawInput::MouseMove(Direction::Right))
                            } else {
                                Some(RawInput::MouseMove(Direction::Left))
                            }
                        } else {
                            if dy > 0 {
                                Some(RawInput::MouseMove(Direction::Up))
                            } else {
                                Some(RawInput::MouseMove(Direction::Down))
                            }
                        }
                    }
                    MouseAction::Press(press_event) => {
                        Some(RawInput::MousePress(press_event))
                    }
                    MouseAction::Release(release_event) => {
                        Some(RawInput::MouseRelease(release_event))
                    }
                }
            }
        }
    }
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
}