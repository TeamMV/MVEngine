use std::ops::Deref;
use crate::geometry::Rect;
use crate::render::ctx::{DrawContext2D, DrawShape};
use crate::styles::{BackgroundRes, Resolve, UiStyle};
use mvutils::state::State;
use mvcore::render::texture::DrawTexture;
use crate::elements::{UiElement, UiElementStub};
use crate::{resolve, styles};
use crate::render::ctx;
use crate::utils::resolve_color;

pub struct ElementBody {
    background_shape: DrawShape,
    border_shape: DrawShape,
}

impl ElementBody {
    pub fn new(background_shape: DrawShape, border_shape: DrawShape) -> Self {
        Self {
            background_shape,
            border_shape,
        }
    }

    pub fn draw(&mut self, elem: &UiElement, ctx: &mut DrawContext2D) {
        let state = elem.state();
        let style = elem.style();
        let bounds = &state.rect;

        let (bgsw, bgsh) = self.background_shape.extent;
        let bg_scale_x = bounds.width() as f32 / bgsw as f32;
        let bg_scale_y = bounds.height() as f32 / bgsh as f32;
        let bg_res = resolve!(elem, background.resource).deref();
        let mut bg_empty = false;
        match bg_res {
            BackgroundRes::Color => {
                let color = resolve!(elem, background.color);
                self.background_shape.set_color(color);
            }
            BackgroundRes::Texture => {
                if !style.background.texture.is_set() {
                    bg_empty = true;
                } else {
                    let tex = resolve!(elem, background.texture).deref().clone();
                    self.background_shape.set_texture(ctx::texture().source(Some(DrawTexture::Texture(tex))));
                }
            }
        }
        if !bg_empty {
            self.background_shape.set_scale(bg_scale_x as i32, bg_scale_y as i32);
            ctx.shape(self.background_shape.clone());
        }

        let (bdsw, bdsd) = self.border_shape.extent;
        let bd_scale_x = bounds.width() as f32 / bdsw as f32;
        let bd_scale_y = bounds.height() as f32 / bdsd as f32;
        let bd_res = resolve!(elem, border.resource).deref();
        let mut bd_empty = false;
        match bd_res {
            BackgroundRes::Color => {
                let color = resolve!(elem, border.color);
                self.border_shape.set_color(color);
            }
            BackgroundRes::Texture => {
                if !style.border.texture.is_set() {
                    bd_empty = true;
                } else {
                    let tex = resolve!(elem, border.texture).deref().clone();
                    self.border_shape.set_texture(ctx::texture().source(Some(DrawTexture::Texture(tex))));
                }
            }
        }
        if !bd_empty {
            self.border_shape.set_scale(bd_scale_x as i32, bd_scale_y as i32);
            ctx.shape(self.border_shape.clone());
        }
    }
}