use crate::color::RgbColor;
use crate::math::vec::Vec4;
use crate::rendering::{InputVertex, RenderContext};
use crate::rendering::texture::Texture;
use crate::ui::context::UiContext;
use crate::ui::elements::{UiElementState, UiElementStub};
use crate::ui::geometry::{Rect, SimpleRect};
use crate::ui::geometry::shape::{Shape, VertexStream};
use crate::ui::rendering::adaptive::AdaptiveFill;
use crate::ui::styles::enums::{BackgroundRes, Direction, Geometry};
use crate::ui::styles::groups::ShapeStyle;
use crate::ui::styles::{DEFAULT_STYLE, UiStyle};
use crate::ui::utils;
use std::ops::Deref;
use crate::ui::elements::components::ElementBody;

pub fn draw_shape_style_at<F: Fn(&UiStyle) -> &ShapeStyle>(
    ctx: &mut impl RenderContext,
    ui_ctx: &UiContext,
    area: &SimpleRect,
    style: &ShapeStyle,
    elem_state: &UiElementState,
    body: &ElementBody,
    map: F,
    crop_area: Option<SimpleRect>,
) {
    let shape = utils::resolve_resolve(&style.shape, elem_state, body, |s| &map(s).shape);
    if !shape.is_none() {
        let shape = shape.unwrap_or_default(&map(&DEFAULT_STYLE).shape);
        let shape = &*shape;

        let resource = utils::resolve_resolve(&style.resource, elem_state, body, |s| &map(s).resource);
        if !resource.is_none() {
            let resource = resource.unwrap_or_default(&map(&DEFAULT_STYLE).resource);
            let resource = &*resource;
            match resource {
                BackgroundRes::Color => {
                    let color = utils::resolve_resolve(&style.color, elem_state, body, |s| &map(s).color);
                    if !color.is_none() {
                        let color = color.unwrap_or_default(&map(&DEFAULT_STYLE).color);
                        match shape {
                            Geometry::Shape(s) => {
                                if let Some(shape) = ui_ctx.resources.resolve_shape(*s) {
                                    draw_shape_color(
                                        ctx,
                                        shape,
                                        color,
                                        area,
                                        crop_area.as_ref().unwrap_or(area),
                                    );
                                }
                            }
                            Geometry::Adaptive(a) => {
                                if let Some(adaptive) = ui_ctx.resources.resolve_adaptive(*a) {
                                    adaptive.draw(
                                        ctx,
                                        area,
                                        AdaptiveFill::Color(color),
                                        ui_ctx,
                                        crop_area.as_ref().unwrap_or(area),
                                    )
                                }
                            }
                        }
                    }
                }
                BackgroundRes::Texture => {
                    let drawable =
                        utils::resolve_resolve(&style.texture, elem_state, body, |s| &map(s).texture);
                    if !drawable.is_none() {
                        let drawable = drawable.unwrap_or_default(&map(&DEFAULT_STYLE).texture);
                        let (tex, uv) = drawable.get_texture_or_default(ui_ctx.resources);
                        match shape {
                            Geometry::Shape(s) => {
                                if let Some(shape) = ui_ctx.resources.resolve_shape(*s) {
                                    draw_shape_textured_at(
                                        ctx,
                                        shape,
                                        tex,
                                        uv,
                                        area,
                                        crop_area.as_ref().unwrap_or(area),
                                    );
                                }
                            }
                            Geometry::Adaptive(a) => {
                                if let Some(adaptive) = ui_ctx.resources.resolve_adaptive(*a) {
                                    let draw = drawable.deref().clone();
                                    adaptive.draw(
                                        ctx,
                                        area,
                                        AdaptiveFill::Drawable(draw),
                                        ui_ctx,
                                        crop_area.as_ref().unwrap_or(area),
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn draw_shape_color(
    ctx: &mut impl RenderContext,
    shape: &Shape,
    color: RgbColor,
    area: &SimpleRect,
    crop: &SimpleRect,
) {
    shape.draw_at(ctx, area, |v| {
        v.has_texture = 0.0;
        v.color = color.as_vec4();
        v.pos.0 = v
            .pos
            .0
            .clamp(crop.x as f32, crop.x as f32 + crop.width as f32);
        v.pos.1 = v
            .pos
            .1
            .clamp(crop.y as f32, crop.y as f32 + crop.height as f32);
    });
}

pub fn draw_shape_textured_at(
    ctx: &mut impl RenderContext,
    shape: &Shape,
    texture: &Texture,
    uv: Vec4,
    area: &SimpleRect,
    crop: &SimpleRect,
) {
    let shape = shape.clone(); //Cannot be asked to do the uv shit here again so the stream it is.
    draw_shape_textured_owned_at(ctx, shape, texture, uv, area, crop);
}

pub fn draw_shape_textured_owned_at(
    ctx: &mut impl RenderContext,
    mut shape: Shape,
    texture: &Texture,
    mut uv: Vec4,
    area: &SimpleRect,
    crop: &SimpleRect,
) {
    //let mut uv = Vec4::default_uv();
    ////uv.x = 0.5;
    //uv.z = 0.5;

    let da = area.create_intersection(crop);

    let a = da.x - area.x;
    let b = da.y - area.y;
    uv.x += a as f32 / area.width as f32;
    uv.y += b as f32 / area.height as f32;
    uv.z *= da.width as f32 / area.width as f32;
    uv.w *= da.height as f32 / area.height as f32;

    shape.stream().texture(texture.id).uv(uv).compute();
    shape.draw_at(ctx, area, |v| {
        //crop to crop area
        v.pos.0 = v.pos.0.clamp(crop.x as f32, (crop.x + crop.width) as f32);
        v.pos.1 = v.pos.1.clamp(crop.y as f32, (crop.y + crop.height) as f32);
    });
}

pub fn draw_shape_textured(
    ctx: &mut impl RenderContext,
    shape: &Shape,
    texture: &Texture,
    uv: Vec4,
    crop: &SimpleRect,
) {
    let shape = shape.clone(); //Cannot be asked to do the uv shit here again so the stream it is.
    draw_shape_textured_owned(ctx, shape, texture, uv, crop);
}

pub fn draw_shape_textured_owned(
    ctx: &mut impl RenderContext,
    mut shape: Shape,
    texture: &Texture,
    uv: Vec4,
    crop: &SimpleRect,
) {
    shape.stream().texture(texture.id).uv(uv).compute();
    let area = &shape.extent;
    shape.draw(ctx, |v| {
        //crop to crop area
        crop_with_uv(v, crop, uv, area);
    });
}

pub fn crop_no_uv(v: &mut InputVertex, crop: &SimpleRect) {
    v.pos.0 = v.pos.0.clamp(crop.x as f32, (crop.x + crop.width) as f32);
    v.pos.1 = v.pos.1.clamp(crop.y as f32, (crop.y + crop.height) as f32);
}

pub fn crop_with_uv(v: &mut InputVertex, crop: &SimpleRect, uv: Vec4, area: &SimpleRect) {
    v.pos.0 = v.pos.0.clamp(crop.x as f32, (crop.x + crop.width) as f32);
    v.pos.1 = v.pos.1.clamp(crop.y as f32, (crop.y + crop.height) as f32);
    if v.has_texture >= 1.0 {
        let x_ratio = if area.width > 0 {
            (v.pos.0 - area.x as f32) / area.width as f32
        } else {
            0.0
        };
        let y_ratio = if area.height > 0 {
            (v.pos.1 - area.y as f32) / area.height as f32
        } else {
            0.0
        };
        v.uv.0 = uv.x + x_ratio * (uv.z - uv.x);
        v.uv.1 = uv.y + y_ratio * (uv.w - uv.y);
    }
}

/// Computes the resulting shape size if this shape will be used to the max extent inside target.
/// - For `Direction::Vertical`: height = target.height, width scaled accordingly.
/// - For `Direction::Horizontal`: width = target.width, height scaled accordingly.
/// - For adaptive shapes: uses `adaptive_ratio` (w / h).
pub fn shape_size<F: Fn(&UiStyle) -> &ShapeStyle>(
    shape_style: &ShapeStyle,
    state: &UiElementState,
    body: &ElementBody,
    ui_ctx: &UiContext,
    target: &SimpleRect,
    direction: Direction,
    adaptive_ratio: f32,
    map: F,
) -> (i32, i32) {
    let shape = utils::resolve_resolve(&shape_style.shape, state, body, |s| &map(s).shape);
    let shape = shape.unwrap_or_default(&map(&DEFAULT_STYLE).shape);

    match &*shape {
        Geometry::Shape(sid) => {
            if let Some(s) = ui_ctx.resources.resolve_shape(*sid) {
                let extent: &SimpleRect = &s.extent;

                match direction {
                    Direction::Vertical => {
                        let scale = target.width as f32 / extent.width as f32;
                        let shape_h = (extent.height as f32 * scale).round() as i32;
                        (target.width, shape_h)
                    }
                    Direction::Horizontal => {
                        let scale = target.height as f32 / extent.height as f32;
                        let shape_w = (extent.width as f32 * scale).round() as i32;
                        (shape_w, target.height)
                    }
                }
            } else {
                (target.width, target.height)
            }
        }
        Geometry::Adaptive(_) => match direction {
            Direction::Vertical => {
                let shape_h = (target.width as f32 / adaptive_ratio).round() as i32;
                (target.width, shape_h)
            }
            Direction::Horizontal => {
                let shape_w = (target.height as f32 * adaptive_ratio).round() as i32;
                (shape_w, target.height)
            }
        },
    }
}