use std::sync::Arc;
use crate::gui::components;
use crate::gui::components::{GuiComponent, GuiElementInfo, GuiTextComponent};
use crate::gui::gui_formats::FormattedString;
use crate::gui::styles::ViewState;
use crate::render::draw2d::Draw2D;
use crate::render::text::{Font, TypeFace};
use crate::resolve;

//------------------------------------
//GuiParagraph
//------------------------------------
pub struct GuiLabel {
    info: GuiElementInfo,
    text: String,
}

impl GuiComponent for GuiLabel {
    fn create() -> Self {
        GuiLabel {
            info: GuiElementInfo::default(),
            text: String::new(),
        }
    }

    fn info(&self) -> &GuiElementInfo {
        &self.info
    }

    fn info_mut(&mut self) -> &mut GuiElementInfo {
        &mut self.info
    }

    fn draw(&mut self, ctx: &mut Draw2D) {
        let view_state = resolve!(self.info, view_state);
        if view_state != ViewState::Gone {
            let font = resolve!(self.info, font)
                .unwrap_or(TypeFace::single(ctx.get_default_font()))
                .regular
                .clone();
            self.info_mut().content_width = font
                .get_metrics(self.text.as_str())
                .width(resolve!(self.info, text_size));
            self.info_mut().content_height = resolve!(self.info, text_size);

            self.info_mut().recalculate_bounds(ctx);

            if view_state == ViewState::Visible {
                components::draw_component_body(ctx, self.info());
                self.draw_line(ctx, self.text.clone(), font);
            }
        } else {
            self.info_mut().content_width = 0;
            self.info_mut().content_height = 0;
        }
    }
}

impl GuiTextComponent for GuiLabel {
    fn get_text(&self) -> &String {
        &self.text
    }
    fn set_text(&mut self, text: String) {
        self.text = text;
    }

    fn draw_line(&mut self, ctx: &mut Draw2D, text: String, font: Arc<Font>) {
        let left = resolve!(self.info, padding_left);
        let bottom = resolve!(self.info, padding_bottom);

        ctx.chroma_tilt(resolve!(self.info, text_chroma_tilt));
        ctx.chroma_compress(resolve!(self.info, text_chroma_compress));
        ctx.get_mut_gradient().copy_of(&resolve!(self.info, text_color));
        ctx.custom_text_origin_rotated(
            resolve!(self.info, text_chroma),
            self.info.x + left,
            self.info.y + bottom,
            self.info.content_height,
            text.as_str(),
            font,
            self.info.rotation,
            self.info.rotation_center.0,
            self.info.rotation_center.1,
        );
    }
}