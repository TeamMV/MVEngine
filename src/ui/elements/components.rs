use crate::color::RgbColor;
use crate::graphics::comp::Drawable;
use crate::rendering::text::Font;
use crate::rendering::{InputVertex, Quad, Transform};
use crate::ui::context::{UiContext, UiResources};
use crate::ui::elements::UiElementStub;
use crate::ui::geometry::shape::Shape;
use crate::ui::rendering::adaptive::{AdaptiveFill, AdaptiveShape};
use crate::ui::rendering::ctx;
use crate::ui::rendering::ctx::DrawContext2D;
use crate::ui::res::err::{ResType, UiResErr};
use crate::ui::res::MVR;
use crate::ui::styles::{BackgroundRes, Dimension, ResolveResult, TextAlign, UiShape, DEFAULT_STYLE};
use crate::{get_adaptive, get_shape, resolve};
use std::marker::PhantomData;
use std::ops::Deref;

#[derive(Clone)]
pub struct ElementBody<E: UiElementStub> {
    _phantom: PhantomData<E>,
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
        let resource = resolve!(elem, background.resource);
        if resolved.is_set() && !resource.is_none() {
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
        let resource = resolve!(elem, border.resource);
        if resolved.is_set() && !resource.is_none() {
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

    fn draw_background_shape(
        mut background_shape: Shape,
        elem: &E,
        ctx: &mut DrawContext2D,
        context: &UiContext,
    ) {
        let state = elem.state();
        let style = elem.style();
        let bounds = &state.rect;

        background_shape.invalidate();
        let (bgsw, bgsh) = background_shape.extent;
        let bg_scale_x = bounds.width() as f32 / bgsw as f32;
        let bg_scale_y = bounds.height() as f32 / bgsh as f32;
        let tmp = resolve!(elem, background.resource);
        if !tmp.is_set() {
            return;
        }
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
            let ui_transform = state.inner_transforms.as_render_transform(state);
            background_shape.set_transform(ui_transform);
            ctx.shape(background_shape);
        }
    }

    fn draw_border_shape(
        mut border_shape: Shape,
        elem: &E,
        ctx: &mut DrawContext2D,
        context: &UiContext,
    ) {
        let state = elem.state();
        let style = elem.style();
        let bounds = &state.rect;

        let (bdsw, bdsd) = border_shape.extent;
        let bd_scale_x = bounds.width() as f32 / bdsw as f32;
        let bd_scale_y = bounds.height() as f32 / bdsd as f32;
        let tmp = resolve!(elem, border.resource);
        if !tmp.is_set() {
            return;
        }
        let bd_res = tmp.unwrap();
        let bd_res = bd_res.deref();
        let mut bd_empty = false;
        match bd_res {
            BackgroundRes::Color => {
                let color = resolve!(elem, border.color);
                if color.is_set() {
                    border_shape.set_color(color.unwrap());
                } else {
                    bd_empty = true;
                }
            }
            BackgroundRes::Texture => {
                if !style.border.texture.is_set() {
                    bd_empty = true;
                } else {
                    let tex = resolve!(elem, border.texture);
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

    fn draw_background_adaptive(
        bg_shape: &AdaptiveShape,
        elem: &E,
        ctx: &mut DrawContext2D,
        context: &UiContext,
    ) {
        let rect = &elem.state().content_rect;
        let res = resolve!(elem, background.resource).unwrap_or(BackgroundRes::Color.into());
        let res = res.deref();
        let fill = match res {
            BackgroundRes::Color => {
                let color = resolve!(elem, background.color).unwrap_or(RgbColor::white());
                AdaptiveFill::Color(color)
            }
            BackgroundRes::Texture => {
                let texture = resolve!(elem, background.texture).unwrap_or(MVR.texture.missing.into());
                let texture = *texture.deref();
                AdaptiveFill::Drawable(Drawable::Texture(texture))
            }
        };
        bg_shape.draw(ctx, rect, fill, context);
    }

    fn draw_border_adaptive(
        bd_shape: &AdaptiveShape,
        elem: &E,
        ctx: &mut DrawContext2D,
        context: &UiContext,
    ) {
        let rect = &elem.state().content_rect;
        let res = resolve!(elem, border.resource).unwrap_or(BackgroundRes::Color.into());
        let res = res.deref();
        let fill = match res {
            BackgroundRes::Color => {
                let color = resolve!(elem, border.color).unwrap_or(RgbColor::white());
                AdaptiveFill::Color(color)
            }
            BackgroundRes::Texture => {
                let texture = resolve!(elem, border.texture).unwrap_or(MVR.texture.missing.into());
                let texture = *texture.deref();
                AdaptiveFill::Drawable(Drawable::Texture(texture))
            }
        };
        bd_shape.draw(ctx, rect, fill, context);
    }
}

#[derive(Clone)]
pub struct TextBody<E: UiElementStub> {
    _phantom: PhantomData<E>,
}

impl<E: UiElementStub> TextBody<E> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData::default(),
        }
    }

    pub fn draw(&self, text: &str, elem: &E, ctx: &mut DrawContext2D, context: &UiContext) {
        let text_align_x = resolve!(elem, text.align_x).unwrap_or(TextAlign::Middle);
        let text_align_y = resolve!(elem, text.align_y).unwrap_or(TextAlign::Middle);
        let font = resolve!(elem, text.font);
        let font = font.unwrap_or(MVR.font.default);
        if let Some(font) = context.resources.resolve_font(font) {
            let color = resolve!(elem, text.color).unwrap_or_default(&DEFAULT_STYLE.text.color);
            let size = resolve!(elem, text.size).unwrap_or_default(&DEFAULT_STYLE.text.size);
            let kerning =
                resolve!(elem, text.kerning).unwrap_or_default(&DEFAULT_STYLE.text.kerning);
            let stretch =
                resolve!(elem, text.stretch).unwrap_or_default(&DEFAULT_STYLE.text.stretch);
            let skew = resolve!(elem, text.skew).unwrap_or_default(&DEFAULT_STYLE.text.skew);
            let shape = Self::create_shape(text, color, size, kerning, stretch, skew, font);
            let state = elem.state();

            let w = shape.extent.0;
            let h = shape.extent.1;

            let text_x = match text_align_x {
                TextAlign::Start => { state.content_rect.x() }
                TextAlign::Middle => { state.content_rect.x() + state.content_rect.width() / 2 - w / 2 }
                TextAlign::End => { state.content_rect.x() + state.content_rect.width() - w }
            };

            let text_y = match text_align_y {
                TextAlign::Start => { state.content_rect.y() }
                TextAlign::Middle => { state.content_rect.y() + state.content_rect.height() / 2 - h / 2 }
                TextAlign::End => { state.content_rect.y() + state.content_rect.height() - h }
            };

            let mut shape = shape.translated(text_x, text_y);
            shape.apply_transformations();
            ctx.shape(shape);
        }
    }

    pub fn create_shape(
        s: &str,
        color: RgbColor,
        size: f32,
        kerning: f32,
        stretch: Dimension<f32>,
        skew: f32,
        font: &Font,
    ) -> Shape {
        let size = size * stretch.height;
        let width = font.get_width(s, size);
        let l = s.len() as f32 - 1f32;
        let width = width * stretch.width + skew * 2f32 + kerning * l;

        let mut triangles = vec![];
        let mut x = 0f32;
        let space_advance = font.get_space_advance(size);
        let mut height = 0;
        for (i, c) in s.char_indices() {
            if c == '\t' {
                x += 6.0 + space_advance;
                continue;
            } else if c == ' ' {
                x += space_advance;
                continue;
            } else if c == '\n' {
                continue;
            }
            let data = font.get_char_data(c, size);
            let vertex = InputVertex {
                transform: Transform::new(),
                pos: (x, 0.0, f32::INFINITY),
                color: color.as_vec4(),
                uv: (0.0, 0.0),
                texture: font.texture().id,
                has_texture: 2.0,
            };
            let mut quad = Quad::from_corner(
                vertex,
                data.uv,
                (data.width * stretch.width, data.size),
                |vertex, (x, y)| vertex.pos = (x, y + data.y_off, vertex.pos.2),
            );
            height = height.max(data.size as i32);
            quad.points[0].transform.translation.x -= skew;
            quad.points[2].transform.translation.x += skew;
            triangles.extend(quad.triangles());
            x += data.width * stretch.width + kerning + skew * 2f32;
        }
        Shape::new_with_extent(triangles, (width as i32, (height as f32 * stretch.height) as i32))
    }
}
