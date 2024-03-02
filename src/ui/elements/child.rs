use std::sync::{Arc, RwLock};
use mvutils::utils::Recover;
use crate::ui::elements::UiElement;

pub enum Child {
    String(String),
    Element(Box<dyn UiElement>),
    DynamicValue(Value<dyn ToString>)
}

impl Child {
    pub fn is_text(&self) -> bool {
        matches!(self, Child::String(_) | Child::DynamicValue(_))
    }

    pub fn is_element(&self) -> bool {
        !self.is_text()
    }

    pub fn as_string(&self) -> String {
        match self {
            Child::String(s) => s.clone(),
            Child::DynamicValue(v) => v.inner.read().recover().to_string(),
            _ => unreachable!(),
        }
    }

    pub fn as_element(&self) -> &Box<dyn UiElement> {
        match self {
            Child::Element(e) => e,
            _ => unreachable!(),
        }
    }
}

pub struct Value<T: ?Sized> {
    inner: Arc<RwLock<T>>,
}