use crate::elements::UiElement;
use mvutils::state::{MappedState, State};
use parking_lot::RwLock;
use std::fmt::Display;
use std::sync::Arc;

pub enum Child {
    String(String),
    Element(Arc<RwLock<UiElement>>),
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

    pub fn as_element(&self) -> Arc<RwLock<UiElement>> {
        match self {
            Child::Element(e) => e.clone(),
            _ => unreachable!(),
        }
    }

    pub fn as_state(&self) -> &State<String> {
        match self {
            Child::State(s) => s,
            _ => unreachable!(),
        }
    }
}

pub trait ToChild {
    fn to_child(self) -> Child;
}

impl<T: ToString> ToChild for T {
    fn to_child(self) -> Child {
        Child::String(self.to_string())
    }
}

impl ToChild for MappedState<String, String> {
    fn to_child(self) -> Child {
        Child::State(self)
    }
}