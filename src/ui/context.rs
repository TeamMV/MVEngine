use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use mvutils::unsafe_utils::DangerousCell;
use crate::color::RgbColor;
use crate::graphics::animation::GlobalAnimation;
use crate::graphics::tileset::TileSet;
use crate::math::vec::Vec4;
use crate::rendering::text::Font;
use crate::rendering::texture::Texture;
use crate::ui::rendering::adaptive::AdaptiveShape;
use crate::ui::rendering::ctx::DrawShape;

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
            inner: Arc::new(DangerousCell::new(inner)),
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
    fn resolve_adaptive(&self, id: usize) -> Option<&AdaptiveShape>;
    fn resolve_texture(&self, id: usize) -> Option<&Texture>;
    fn resolve_font(&self, id: usize) -> Option<&Font>;
    fn resolve_tile(&self, id: usize, index: usize) -> Option<(&Texture, Vec4)>;
    fn resolve_tileset(&self, id: usize) -> Option<&TileSet>;
    fn resolve_animation(&self, id: usize) -> Option<&GlobalAnimation>;

    fn tick_all_animations(&self);
}

pub struct InnerContext {
    pub(crate) resources: &'static dyn UiResources
}