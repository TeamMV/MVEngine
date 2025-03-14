use std::marker::PhantomData;
use std::ops::Deref;
use crate::ui::geometry::Rect;
use crate::ui::styles::{BackgroundRes, Dimension, Resolve, ResolveResult, UiShape, UiStyle, DEFAULT_STYLE};
use mvutils::state::State;
use crate::{get_adaptive, get_shape, resolve};
use crate::color::RgbColor;
use crate::math::vec::Vec2;
use crate::rendering::control::RenderController;
use crate::rendering::{InputVertex, Quad, Transform};
use crate::rendering::text::Font;
use crate::ui::elements::{UiElement, UiElementStub};
use crate::ui::context::{UiContext, UiResources};
use crate::ui::rendering::adaptive::AdaptiveShape;
use crate::ui::rendering::ctx;
use crate::ui::rendering::ctx::{DrawContext2D, DrawShape};
use crate::ui::res::err::{UiResErr, ResType, UiResult};
use crate::ui::res::MVR;

#[derive(Clone)]
pub struct ElementBody<E: UiElementStub> {
    _phantom: PhantomData<E>
}

impl<E: UiElementStub> ElementBody<E> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData::default(),
        }
    }

    pub fn draw(&mut self, elem: &E, ctx: &mut DrawContext2D, context: &UiContext) {
        let res = context.resources;

        let resolved = resolve!(elem, background.shape);
        if resolved.is_set() {
            let resolved = resolved.unwrap();
            match resolved.deref().clone() {
                UiShape::Shape(s) => {
                    let shape = get_shape!(res, s).ok();
                    if let Some(shape) = shape {
                        Self::draw_background_shape(shape.clone(), elem, ctx, context);
                    }
                }
                UiShape::Adaptive(a) => {
                    let shape = get_adaptive!(res, a).ok();
                    if let Some(shape) = shape {
                        Self::draw_background_adaptive(shape, elem, ctx, context);
                    }
                }
            }
        }

        let resolved = resolve!(elem, border.shape);
        if resolved.is_set() {
            let resolved = resolved.unwrap();
            match resolved.deref().clone() {
                UiShape::Shape(s) => {
                    let shape = get_shape!(res, s).ok();
                    if let Some(shape) = shape {
                        Self::draw_border_shape(shape.clone(), elem, ctx, context);
                    }
                }
                UiShape::Adaptive(a) => {
                    let shape = get_adaptive!(res, a).ok();
                    if let Some(shape) = shape {
                        Self::draw_border_adaptive(shape, elem, ctx, context);
                    }
                }
            }
        }
    }

    fn draw_background_shape(mut background_shape: DrawShape, elem: &E, ctx: &mut DrawContext2D, context: &UiContext) {
        let state = elem.state();
        let style = elem.style();
        let bounds = &state.rect;

        background_shape.compute_extent();
        let (bgsw, bgsh) = background_shape.extent;
        let bg_scale_x = bounds.width() as f32 / bgsw as f32;
        let bg_scale_y = bounds.height() as f32 / bgsh as f32;
        let tmp = resolve!(elem, background.resource);
        if !tmp.is_set() { return; }
        let bg_res = tmp.unwrap();
        let bg_res = bg_res.deref();
        let mut bg_empty = false;
        match bg_res {
            BackgroundRes::Color => {
                let color = resolve!(elem, background.color);
                if color.is_set() {
                    background_shape.set_color(color.unwrap());
                } else {
                    bg_empty = true;
                }
            }
            BackgroundRes::Texture => {
                if !style.background.texture.is_set() {
                    bg_empty = true;
                } else {
                    let tex = resolve!(elem, background.texture);
                    if let ResolveResult::Value(tex) = tex {
                        let tex = tex.deref().clone();
                        if let Some(tex) = context.resources.resolve_texture(tex) {
                            background_shape.set_texture(ctx::texture().source(Some(tex.clone())));
                        } else {
                            bg_empty = true;
                        }
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

    fn draw_border_shape(mut border_shape: DrawShape, elem: &E, ctx: &mut DrawContext2D, context: &UiContext) {
        let state = elem.state();
        let style = elem.style();
        let bounds = &state.rect;

        let (bdsw, bdsd) = border_shape.extent;
        let bd_scale_x = bounds.width() as f32 / bdsw as f32;
        let bd_scale_y = bounds.height() as f32 / bdsd as f32;
        let tmp = resolve!(elem, border.resource);
        if !tmp.is_set() { return; }
        let bd_res = tmp.unwrap();
        let bd_res = bd_res.deref();
        let mut bd_empty = false;
        match bd_res {
            BackgroundRes::Color => {
                let color = resolve!(elem, background.color);
                if color.is_set() {
                    border_shape.set_color(color.unwrap());
                } else {
                    bd_empty = true;
                }
            }
            BackgroundRes::Texture => {
                if !style.background.texture.is_set() {
                    bd_empty = true;
                } else {
                    let tex = resolve!(elem, background.texture);
                    if let ResolveResult::Value(tex) = tex {
                        let tex = tex.deref().clone();
                        if let Some(tex) = context.resources.resolve_texture(tex) {
                            border_shape.set_texture(ctx::texture().source(Some(tex.clone())));
                        } else {
                            bd_empty = true;
                        }
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

    fn draw_background_adaptive(bg_shape: &AdaptiveShape, elem: &E, ctx: &mut DrawContext2D, context: &UiContext) {}

    fn draw_border_adaptive(bd_shape: &AdaptiveShape, elem: &E, ctx: &mut DrawContext2D, context: &UiContext) {}
}

#[derive(Clone)]
pub struct TextBody<E: UiElementStub> {
    _phantom: PhantomData<E>
}

impl<E: UiElementStub> TextBody<E> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData::default(),
        }
    }

    pub fn draw(&self, text: &str, elem: &E, ctx: &mut DrawContext2D, context: &UiContext) {
        let font = resolve!(elem, text.font);
        let font = font.unwrap_or(MVR.font.default);
        if let Some(font) = context.resources.resolve_font(font) {
            let color = resolve!(elem, text.color).unwrap_or_default(&DEFAULT_STYLE.text.color);
            let size = resolve!(elem, text.size).unwrap_or_default(&DEFAULT_STYLE.text.size);
            let kerning = resolve!(elem, text.kerning).unwrap_or_default(&DEFAULT_STYLE.text.kerning);
            let stretch = resolve!(elem, text.stretch).unwrap_or_default(&DEFAULT_STYLE.text.stretch);
            let skew = resolve!(elem, text.skew).unwrap_or_default(&DEFAULT_STYLE.text.skew);
            let shape = Self::create_shape(text, color, size, kerning, stretch, skew, font);
            let state = elem.state();
            let shape = shape.translated(state.content_rect.x(), state.content_rect.y());
            ctx.shape(shape);
        }
    }

    pub fn create_shape(s: &str, color: RgbColor, size: f32, kerning: f32, stretch: Dimension<f32>, skew: f32, font: &Font) -> DrawShape {
        let width = font.get_width(s, size);
        let l = s.len() as f32 - 1f32;
        let width = width * stretch.width + skew * 2f32 + kerning * l;

        let mut triangles = vec![];
        let mut x = 0f32;
        for (i, c) in s.char_indices() {
            let data = font.get_char_data(c, size);
            let vertex = InputVertex {
                transform: Transform::new(),
                pos: (x, 0.0, 0.0),
                color: color.as_vec4(),
                uv: (0.0, 0.0),
                texture: 0,
                has_texture: 2.0,
            };
            let mut quad = Quad::from_corner(vertex, data.uv, (data.width * stretch.width, size), |vertex, (x, y)| vertex.transform.translation = Vec2::new(x, y));
            quad.points[0].transform.translation.x -= skew;
            quad.points[2].transform.translation.x += skew;
            triangles.extend(quad.triangles());
            x += data.width * stretch.width + kerning + skew * 2f32;
        }
        DrawShape {
            triangles,
            extent: (width as i32, (size * stretch.height) as i32),
        }
    }
}