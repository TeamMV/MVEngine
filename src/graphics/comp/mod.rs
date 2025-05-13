pub mod parse;
pub mod rig;

use crate::graphics::comp::parse::MRFParser;
use crate::math::vec::Vec4;
use crate::rendering::texture::Texture;
use crate::ui::context::UiResources;
use crate::ui::res::MVR;
use mvutils::Savable;
use crate::ui::styles::BasicInterpolatable;

#[derive(Savable)]
pub struct CompositeSprite {
    parts: Vec<Drawable>,
}

impl CompositeSprite {
    pub fn from_expr_and_resources(expr: &str, resources: Vec<Drawable>) -> Result<Self, String> {
        let parser = MRFParser::parse(expr)?;
        Ok(Self {
            parts: resources,
        })
    }

    pub fn get_part_drawable(&self, index: usize) -> &Drawable {
        self.parts.get(index).unwrap()
    }
}

#[derive(Clone, Savable)]
pub enum Drawable {
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
        }
    }

    pub fn get_texture_or_default(&self, res: &'static (impl UiResources + ?Sized)) -> (&'static Texture, Vec4) {
        let tex = match self {
            Drawable::Texture(t) => res.resolve_texture(*t).map(|t| (t, Vec4::default_uv())),
            Drawable::Animation(a) => res.resolve_animation(*a).map(|a| a.get_current()),
            Drawable::TileSet(ts, idx) => res.resolve_tile(*ts, *idx),
        };
        if let Some(tex) = tex {
            tex
        } else {
            let texture = MVR.resolve_texture(MVR.texture.missing).expect("Cannot get missing texture! Make sure to use the r! macro to create your resource struct");
            (texture, Vec4::default_uv())
        }
    }
}
