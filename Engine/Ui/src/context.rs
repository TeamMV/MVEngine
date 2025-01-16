use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use mvutils::unsafe_utils::DangerousCell;
use mvcore::color::RgbColor;
use mvcore::ToAD;
use crate::render::ctx::DrawShape;

#[derive(Clone)]
pub struct UiContext {
    inner: Arc<DangerousCell<InnerContext>>
}

impl UiContext {
    pub(crate) fn new(resources: &'static dyn UiResources) -> Self {
        let inner = InnerContext {
            resources,
        };
        Self {
            inner: inner.to_ad(),
        }
    }
}

impl Deref for UiContext {
    type Target = InnerContext;

    fn deref(&self) -> &Self::Target {
        self.inner.get()
    }
}

impl DerefMut for UiContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.get_mut()
    }
}

pub trait UiResources {
    fn resolve_color(&self, id: usize) -> Option<&RgbColor>;
    fn resolve_shape(&self, id: usize) -> Option<&DrawShape>;
}

pub struct InnerContext {
    pub(crate) resources: &'static dyn UiResources
}