use crate::color::RgbColor;
use crate::math::vec::Vec4;
use crate::rendering::texture::Texture;
use crate::rendering::{InputVertex, Quad, RenderContext};
use crate::ui::context::UiResources;
use crate::ui::geometry::{shape, Rect, SimpleRect};
use crate::ui::res::MVR;
use mvutils::Savable;
use std::str::FromStr;
use crate::ui::geometry::shape::shapes;

pub mod animation;
pub mod comp;
pub mod particle;
pub mod tileset;

#[derive(Clone, Savable)]
pub enum Drawable {
    Color(usize),
    Texture(usize),
    Animation(usize),
    TileSet(usize, usize),
}

impl Drawable {
    pub fn missing() -> Drawable {
        Drawable::Texture(MVR.texture.missing)
    }
}

impl Drawable {
    pub fn get_texture(
        &self,
        res: &'static (impl UiResources + ?Sized),
    ) -> Option<(&'static Texture, Vec4)> {
        match self {
            Drawable::Texture(t) => res.resolve_texture(*t).map(|t| (t, Vec4::default_uv())),
            Drawable::Animation(a) => res.resolve_animation(*a).map(|a| a.get_current()),
            Drawable::TileSet(ts, idx) => res.resolve_tile(*ts, *idx),
            _ => None,
        }
    }

    pub fn get_texture_or_default(
        &self,
        res: &'static (impl UiResources + ?Sized),
    ) -> (&'static Texture, Vec4) {
        let tex = match self {
            Drawable::Texture(t) => res.resolve_texture(*t).map(|t| (t, Vec4::default_uv())),
            Drawable::Animation(a) => res.resolve_animation(*a).map(|a| a.get_current()),
            Drawable::TileSet(ts, idx) => res.resolve_tile(*ts, *idx),
            _ => None,
        };
        if let Some(tex) = tex {
            tex
        } else {
            let texture = MVR.resolve_texture(MVR.texture.missing).expect("Cannot get missing texture! Make sure to use the r! macro to create your resource struct");
            (texture, Vec4::default_uv())
        }
    }

    pub fn draw(
        &self,
        ctx: &mut impl RenderContext,
        area: Rect,
        r: &'static (impl UiResources + ?Sized),
        crop: &SimpleRect
    ) {
        match self {
            _ => {
                // texture, tileset, anination are all based on texture
                let trans = area.get_transform();
                let (tex, uv) = self.get_texture_or_default(r);
                let rect = shapes::rectangle0(area.x(), area.y(), area.width(), area.height());
                shape::utils::draw_shape_textured_owned(ctx, rect, tex, uv, crop);
            }
        }
    }
}

impl FromStr for Drawable {
    type Err = String;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        // good implementation
        todo!()
    }
}
