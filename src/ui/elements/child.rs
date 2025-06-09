use crate::ui::elements::{Element, UiElement};
use itertools::Itertools;
use mvutils::state::MappedState;
use mvutils::unsafe_utils::DangerousCell;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub enum Child {
    String(String),
    Element(Rc<DangerousCell<UiElement>>),
    State(MappedState<String, String>),
    Iterator(Vec<Child>),
}

impl fmt::Debug for Child {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variant_name = match self {
            Child::String(_) => "String",
            Child::Element(_) => "Element",
            Child::State(_) => "State",
            Child::Iterator(_) => "Iterator",
        };
        write!(f, "Child::{}", variant_name)
    }
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

impl ToChild for Element {
    fn to_child(self) -> Child {
        Child::Element(self)
    }
}

// To avoid specialization
pub trait ToChildFromIterator {
    fn to_child(self) -> Child;
}

impl<T: Iterator<Item = C>, C: ToChild> ToChildFromIterator for T {
    fn to_child(self) -> Child {
        Child::Iterator(self.map(|x| x.to_child()).collect_vec())
    }
}
