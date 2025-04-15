pub mod parse;
pub mod rig;

use crate::graphics::comp::parse::MRFParser;
use crate::graphics::comp::rig::Rig;
use crate::math::vec::Vec4;
use crate::rendering::texture::Texture;
use crate::ui::context::UiResources;
use std::marker::PhantomData;

pub struct CompositeSprite {
    parts: Vec<Drawable>,
    rig: Rig,
}

impl CompositeSprite {
    pub fn from_expr_and_resources(expr: &str, resources: Vec<Drawable>) -> Result<Self, String> {
        let parser = MRFParser::parse(expr)?;
        let rig = Rig::from_parsed(parser);
        Ok(Self {
            parts: resources,
            rig,
        })
    }

    pub fn rig(&self) -> &Rig {
        &self.rig
    }

    pub fn rig_mut(&mut self) -> &mut Rig {
        &mut self.rig
    }

    pub fn get_part_drawable(&self, index: usize) -> &Drawable {
        self.parts.get(index).unwrap()
    }
}

pub enum Drawable {
    Texture(usize),
    Animation(usize),
    TileSet(usize, usize),
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
}
