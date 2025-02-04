use crate::ui::elements::UiElement;
use mvutils::state::{MappedState, State};
use parking_lot::RwLock;
use std::fmt::Display;
use std::rc::Rc;
use std::sync::Arc;
use mvutils::unsafe_utils::DangerousCell;

#[derive(Clone)]
pub enum Child {
    String(String),
    Element(Rc<DangerousCell<UiElement>>),
    State(MappedState<String, String>),
}

impl Child {
    pub fn is_text(&self) -> bool {
        matches!(self, Child::String(_))
    }

    pub fn is_element(&self) -> bool {
        matches!(self, Child::Element(_))
    }

    pub fn is_state(&self) -> bool {
        matches!(self, Child::State(_))
    }

    pub fn as_string(&self) -> String {
        match self {
            Child::String(s) => s.clone(),
            _ => unreachable!(),
        }
    }

    pub fn as_element(&self) -> Rc<DangerousCell<UiElement>> {
        match self {
            Child::Element(e) => e.clone(),
            _ => unreachable!(),
        }
    }

    pub fn as_state(&self) -> &MappedState<String, String> {
        match self {
            Child::State(s) => s,
            _ => unreachable!(),
        }
    }
}

pub trait ToChild {
    fn to_child(self) -> Child;
}

impl ToChild for String {
    fn to_child(self) -> Child {
        Child::String(self)
    }
}

impl ToChild for &str {
    fn to_child(self) -> Child {
        Child::String(self.to_string())
    }
}

impl ToChild for MappedState<String, String> {
    fn to_child(self) -> Child {
        Child::State(self)
    }
}