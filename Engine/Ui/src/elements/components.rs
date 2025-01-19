use std::marker::PhantomData;
use std::ops::Deref;
use crate::geometry::Rect;
use crate::render::ctx::{DrawContext2D, DrawShape};
use crate::styles::{BackgroundRes, Resolve, UiShape, UiStyle};
use mvutils::state::State;
use mvcore::render::texture::DrawTexture;
use crate::elements::{UiElement, UiElementStub};
use crate::{get_adaptive, get_shape, resolve, styles};
use crate::context::{UiContext, UiResources};
use crate::render::adaptive::AdaptiveShape;
use crate::render::ctx;
use crate::res::err::{UiResErr, ResType, UiResult};
use crate::res::MVR;

#[derive(Clone)]
pub struct ElementBody<E: UiElementStub> {
    context: UiContext,
    _phantom: PhantomData<E>
}

impl<E: UiElementStub> ElementBody<E> {
    pub fn new(context: UiContext) -> Self {
        Self {
            context: context.clone(),
            _phantom: PhantomData::default(),
        }
    }

    pub fn draw(&mut self, elem: &E, ctx: &mut DrawContext2D) {
        let res = self.context.resources;
        let style = elem.style();

        if style.background.shape.is_set() || style.background.shape.is_auto() {
            let resolved = resolve!(elem, background.shape);
            match resolved.deref().clone() {
                UiShape::Shape(s) => {
                    let shape = get_shape!(res, s).ok();
                    if let Some(shape) = shape {
                        Self::draw_background_shape(shape.clone(), elem, ctx);
                    }
                }
                UiShape::Adaptive(a) => {
                    let shape = get_adaptive!(res, a).ok();
                    if let Some(shape) = shape {
                        Self::draw_background_adaptive(shape, elem, ctx);
                    }
                }
            }
        }

        if style.border.shape.is_set() || style.border.shape.is_auto() {
            let resolved = resolve!(elem, border.shape);
            match resolved.deref().clone() {
                UiShape::Shape(s) => {
                    let shape = get_shape!(res, s).ok();
                    if let Some(shape) = shape {
                        Self::draw_border_shape(shape.clone(), elem, ctx);
                    }
                }
                UiShape::Adaptive(a) => {
                    let shape = get_adaptive!(res, a).ok();
                    if let Some(shape) = shape {
                        Self::draw_border_adaptive(shape, elem, ctx);
                    }
                }
            }
        }
    }

    fn draw_background_shape(mut background_shape: DrawShape, elem: &E, ctx: &mut DrawContext2D) {
        let state = elem.state();
        let style = elem.style();
        let bounds = &state.rect;

        background_shape.compute_extent();
        let (bgsw, bgsh) = background_shape.extent;
        let bg_scale_x = bounds.width() as f32 / bgsw as f32;
        let bg_scale_y = bounds.height() as f32 / bgsh as f32;
        let tmp = resolve!(elem, background.resource);
        let bg_res = tmp.deref();
        let mut bg_empty = false;
        match bg_res {
            BackgroundRes::Color => {
                let color = resolve!(elem, background.color);
                background_shape.set_color(color);
            }
            BackgroundRes::Texture => {
                if !style.background.texture.is_set() {
                    bg_empty = true;
                } else {
                    let tex = resolve!(elem, background.texture).deref().clone();
                    if let Some(tex) = MVR.resolve_texture(tex) {
                        background_shape.set_texture(ctx::texture().source(Some(DrawTexture::Texture(tex.clone()))));
                    } else {
                        bg_empty = true;
                    }
                }
            }
        }
        if !bg_empty {
            background_shape.set_translate(state.rect.x(), state.rect.y());
            background_shape.apply_transformations();
            background_shape.set_origin(state.rect.x(), state.rect.y());
            background_shape.set_scale(bg_scale_x, bg_scale_y);
            background_shape.apply_transformations();
            ctx.shape(background_shape);
        }
    }

    fn draw_border_shape(mut border_shape: DrawShape, elem: &E, ctx: &mut DrawContext2D) {
        let state = elem.state();
        let style = elem.style();
        let bounds = &state.rect;

        let (bdsw, bdsd) = border_shape.extent;
        let bd_scale_x = bounds.width() as f32 / bdsw as f32;
        let bd_scale_y = bounds.height() as f32 / bdsd as f32;
        let tmp = resolve!(elem, border.resource);
        let bd_res = tmp.deref();
        let mut bd_empty = false;
        match bd_res {
            BackgroundRes::Color => {
                let color = resolve!(elem, border.color);
                border_shape.set_color(color);
            }
            BackgroundRes::Texture => {
                if !style.border.texture.is_set() {
                    bd_empty = true;
                } else {
                    let tex = resolve!(elem, border.texture).deref().clone();
                    if let Some(tex) = MVR.resolve_texture(tex) {
                        border_shape.set_texture(ctx::texture().source(Some(DrawTexture::Texture(tex.clone()))));
                    } else {
                        bd_empty = true;
                    }
                }
            }
        }
        if !bd_empty {
            border_shape.set_scale(bd_scale_x, bd_scale_y);
            ctx.shape(border_shape);
        }
    }

    fn draw_background_adaptive(bg_shape: &AdaptiveShape, elem: &E, ctx: &mut DrawContext2D) {}

    fn draw_border_adaptive(bd_shape: &AdaptiveShape, elem: &E, ctx: &mut DrawContext2D) {}
}