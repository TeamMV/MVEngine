use crate::render::draw2d::DrawContext2D;
use crate::ui::attributes::Attributes;
use crate::ui::elements::{UiElement, UiElementCallbacks, UiElementState};
use crate::ui::styles::{Dimension, UiStyle};
use mvutils::utils::Recover;
use parking_lot::RwLock;
use std::ops::Deref;
use std::sync::Arc;

pub enum Child {
    String(String),
    Element(Arc<RwLock<dyn UiElement>>),
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

    pub fn as_element(&self) -> Arc<RwLock<dyn UiElement>> {
        match self {
            Child::Element(e) => e.clone(),
            _ => unreachable!(),
        }
    }
}

pub struct Value<T: ?Sized> {
    inner: Arc<RwLock<T>>,
}
