use crate::graphics::Drawable;
use crate::math::vec::Vec2;
use crate::rendering::RenderContext;
use crate::ui::context::UiResources;
use crate::ui::geometry::Rect;
use crate::ui::styles::enums::Geometry;

#[derive(Clone)]
pub struct ParticleSystem {
    position: Vec2,
}

pub struct Particle {
    area: Rect,
    velocity: Vec2,
    scale: Vec2,
    drawable: Drawable,
    shape: Geometry,
}

impl Particle {
    pub fn draw(
        &self,
        ctx: &mut impl RenderContext,
        area: Rect,
        r: &'static (impl UiResources + ?Sized),
    ) {
        match self.shape {
            Geometry::Shape(s) => if let Some(res_shape) = r.resolve_shape(s) {},
            Geometry::Adaptive(a) => if let Some(adaptive) = r.resolve_adaptive(a) {},
        }
    }
}
