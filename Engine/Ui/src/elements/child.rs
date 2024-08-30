use crate::elements::UiElement;
use parking_lot::RwLock;
use std::sync::Arc;

pub enum Child {
    String(String),
    Element(Arc<RwLock<UiElement>>),
    DynamicValue(Value<dyn ToString>),
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
            Child::DynamicValue(v) => v.inner.read().to_string(),
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

pub struct Value<T: ?Sized> {
    inner: Arc<RwLock<T>>,
}
