pub mod parse;
pub mod rig;

use crate::graphics::comp::parse::MRFParser;
use crate::graphics::Drawable;
use mvutils::Savable;
#[derive(Savable)]
pub struct CompositeSprite {
    parts: Vec<Drawable>,
}

impl CompositeSprite {
    pub fn from_expr_and_resources(expr: &str, resources: Vec<Drawable>) -> Result<Self, String> {
        //TODO: parse mrf files
        let _parser = MRFParser::parse(expr)?;
        Ok(Self { parts: resources })
    }

    pub fn get_part_drawable(&self, index: usize) -> &Drawable {
        self.parts.get(index).unwrap()
    }
}
