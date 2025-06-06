pub mod parse;
pub mod rig;

use std::str::FromStr;
use crate::graphics::comp::parse::MRFParser;
use crate::ui::context::UiResources;
use mvutils::Savable;
use crate::graphics::Drawable;
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