use std::ops::Deref;
use crate::color::RgbColor;
use crate::math::vec::Vec4;
use crate::rendering::RenderContext;
use crate::rendering::texture::Texture;
use crate::ui::context::UiContext;
use crate::ui::elements::UiElementStub;
use crate::ui::geometry::shape::{Shape, VertexStream};
use crate::ui::geometry::SimpleRect;
use crate::ui::rendering::adaptive::AdaptiveFill;
use crate::ui::styles::enums::{BackgroundRes, Geometry};
use crate::ui::styles::groups::ShapeStyle;
use crate::ui::styles::{UiStyle, DEFAULT_STYLE};
use crate::ui::utils;

pub fn draw_shape_style_at<E: UiElementStub + 'static, F: Fn(&UiStyle) -> &ShapeStyle>(ctx: &mut impl RenderContext, ui_ctx: &UiContext, area: &SimpleRect, style: &ShapeStyle, elem: &E, map: F) {
    let shape = utils::resolve_resolve(&style.shape, elem, |s| &map(s).shape);
    if !shape.is_none() {
        let shape = shape.unwrap_or_default(&map(&DEFAULT_STYLE).shape);
        let shape = &*shape;

        let resource = utils::resolve_resolve(&style.resource, elem, |s| &map(s).resource);
        if !resource.is_none() {
            let resource = resource.unwrap_or_default(&map(&DEFAULT_STYLE).resource);
            let resource = &*resource;
            match resource {
                BackgroundRes::Color => {
                    let color = utils::resolve_resolve(&style.color, elem, |s| &map(s).color);
                    if !color.is_none() {
                        let color = color.unwrap_or_default(&map(&DEFAULT_STYLE).color);
                        match shape {
                            Geometry::Shape(s) => {
                                if let Some(shape) = ui_ctx.resources.resolve_shape(*s) {
                                    draw_shape_color(ctx, shape, color, area);
                                }
                            }
                            Geometry::Adaptive(a) => {
                                if let Some(adaptive) = ui_ctx.resources.resolve_adaptive(*a) {
                                    adaptive.draw(ctx, area, AdaptiveFill::Color(color), ui_ctx)
                                }
                            }
                        }
                    }
                }
                BackgroundRes::Texture => {
                    let drawable = utils::resolve_resolve(&style.texture, elem, |s| &map(s).texture);
                    if !drawable.is_none() {
                        let drawable = drawable.unwrap_or_default(&map(&DEFAULT_STYLE).texture);
                        let (tex, uv) = drawable.get_texture_or_default(ui_ctx.resources);
                        match shape {
                            Geometry::Shape(s) => {
                                if let Some(shape) = ui_ctx.resources.resolve_shape(*s) {
                                    draw_shape_textured(ctx, shape, tex, uv, area);
                                }
                            }
                            Geometry::Adaptive(a) => {
                                if let Some(adaptive) = ui_ctx.resources.resolve_adaptive(*a) {
                                    let draw = drawable.deref().clone();
                                    adaptive.draw(ctx, area, AdaptiveFill::Drawable(draw), ui_ctx)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn draw_shape_color(ctx: &mut impl RenderContext, shape: &Shape, color: RgbColor, area: &SimpleRect) {
    shape.draw_at(ctx, area, |v| {
        v.has_texture = 0.0;
        v.color = color.as_vec4();
    });
}

pub fn draw_shape_textured(ctx: &mut impl RenderContext, shape: &Shape, texture: &Texture, uv: Vec4, area: &SimpleRect) {
    let mut shape = shape.clone(); //Cannot be asked to do the uv shit here again so the stream it is.
    shape.stream()
        .texture(texture.id)
        .uv(uv)
        .compute();
    shape.draw_at(ctx, area, |_| {});
}