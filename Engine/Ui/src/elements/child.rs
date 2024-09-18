use std::fmt::Display;
use crate::elements::UiElement;
use parking_lot::RwLock;
use std::sync::Arc;
use mvutils::state::State;

pub enum Child {
    String(String),
    Element(Arc<RwLock<UiElement>>),
}

impl Child {
    pub fn is_text(&self) -> bool {
        matches!(self, Child::String(_))
    }

    pub fn is_element(&self) -> bool {
        !self.is_text()
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
}
