pub mod parse;
pub mod rig;

use crate::graphics::comp::parse::parser::MRFParser;
use crate::graphics::comp::rig::Rig;
use crate::rendering::RenderContext;
use crate::ui::context::UiResources;
use crate::ui::geometry::SimpleRect;
use mvutils::Savable;

#[derive(Savable, Clone)]
pub struct CompositeSprite {
    pub rig: Rig,
}

impl CompositeSprite {
    pub fn from_rig(expr: &str) -> Result<Self, String> {
        let parsed_rig = MRFParser::parse(expr)?;
        let rig = Rig::from_parsed(parsed_rig)?;

        Ok(Self { rig })
    }

    pub fn add_drawable(&mut self, part: &str, drawable: usize) {
        if let Some(part) = self.rig.skeleton.parts.get(part) {
            part.write().set_drawable(Some(drawable));
        }
    }

    pub fn draw(
        &self,
        ctx: &mut impl RenderContext,
        r: &'static (impl UiResources + ?Sized),
        area: &SimpleRect,
    ) {
        self.rig.draw(ctx, r, area);
    }
}
