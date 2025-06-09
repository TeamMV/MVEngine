use crate::math::vec::Vec4;
use crate::rendering::texture::Texture;
use crate::rendering::{InputVertex, Quad, RenderContext};
use crate::ui::context::UiResources;
use crate::ui::geometry::Rect;
use crate::ui::res::MVR;
use mvutils::Savable;
use std::str::FromStr;

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
    ) {
        let controller = ctx.controller();
        match self {
            _ => {
                // texture, tileset, anination are all based on texture
                let trans = area.get_transform();
                let (tex, uv) = self.get_texture_or_default(r);
                let vertex = InputVertex {
                    transform: trans,
                    pos: (area.x() as f32, area.y() as f32, 0.0),
                    color: Default::default(),
                    uv: (0.0, 0.0),
                    texture: tex.id,
                    has_texture: 1.0,
                };
                let quad = Quad::from_corner(
                    vertex,
                    uv,
                    (area.width() as f32, area.height() as f32),
                    |v, (x, y)| {
                        v.pos.0 = x;
                        v.pos.1 = y;
                    },
                );
                controller.push_quad(quad);
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
