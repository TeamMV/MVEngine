use crate::color::RgbColor;
use crate::graphics::animation::GlobalAnimation;
use crate::graphics::comp::CompositeSprite;
use crate::graphics::tileset::TileSet;
use crate::graphics::Drawable;
use crate::math::vec::Vec4;
use crate::rendering::text::Font;
use crate::rendering::texture::Texture;
use crate::ui::geometry::shape::Shape;
use crate::ui::rendering::adaptive::AdaptiveShape;
use crate::ui::styles::enums::Geometry;
use mvutils::unsafe_utils::DangerousCell;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

#[derive(Clone)]
pub struct UiContext {
    inner: Arc<DangerousCell<InnerContext>>,
}

impl UiContext {
    pub(crate) fn new(resources: &'static dyn UiResources) -> Self {
        let inner = InnerContext { resources };
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
    fn resolve_shape(&self, id: usize) -> Option<&Shape>;
    fn resolve_adaptive(&self, id: usize) -> Option<&AdaptiveShape>;
    fn resolve_texture(&self, id: usize) -> Option<&Texture>;
    fn resolve_font(&self, id: usize) -> Option<&Font>;
    fn resolve_tile(&self, id: usize, index: usize) -> Option<(&Texture, Vec4)>;
    fn resolve_tileset(&self, id: usize) -> Option<&TileSet>;
    fn resolve_animation(&self, id: usize) -> Option<&GlobalAnimation>;
    fn resolve_composite(&self, id: usize) -> Option<&CompositeSprite>;
    fn resolve_drawable(&self, id: usize) -> Option<&Drawable>;
    fn resolve_geometry(&self, id: usize) -> Option<&Geometry>;

    fn tick_all_animations(&self);
}

pub struct InnerContext {
    pub(crate) resources: &'static dyn UiResources,
}
