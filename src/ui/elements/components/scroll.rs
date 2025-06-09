use crate::ui::res::err::UiResErr;
use crate::ui::res::err::ResType;
use std::ops::Deref;
use crate::ui::styles::{ResolveResult, DEFAULT_STYLE};
use crate::color::RgbColor;
use crate::ui::context::UiContext;
use crate::ui::elements::{UiElementState, UiElementStub};
use crate::ui::geometry::{Rect, SimpleRect};
use crate::ui::rendering::ctx;
use crate::ui::rendering::ctx::DrawContext2D;
use mvutils::lazy;
use crate::{get_adaptive, get_shape, resolve};
use crate::graphics::Drawable;
use crate::ui::geometry::shape::Shape;
use crate::ui::rendering::adaptive::{AdaptiveFill, AdaptiveShape};
use crate::ui::styles::enums::{BackgroundRes, Geometry};

lazy! {
    pub static OUTER_COLOR: RgbColor = RgbColor::new([150, 150, 150, 255]);
    pub static INNER_COLOR: RgbColor = RgbColor::new([87, 87, 87, 255]);
}

#[derive(Clone)]
pub struct ScrollBars {}

impl ScrollBars {
    pub fn draw<E: UiElementStub + 'static>(
        &mut self,
        elem: &E,
        ctx: &mut DrawContext2D,
        context: &UiContext,
        crop_area: &SimpleRect
    ) {
        let res = context.resources;
        let state = elem.state();
        
        let bar_extent = resolve!(elem, scrollbar.size)
            .unwrap_or_default_or_percentage(&DEFAULT_STYLE.scrollbar.size, state.parent.clone(), |s| s.width(), state);
        
        
        if state.scroll_x.available {
            let resolved = resolve!(elem, scrollbar.track.shape);
            let resource = resolve!(elem, scrollbar.track.resource);
            if resolved.is_set() && !resource.is_none() {
                let mut rect = state.content_rect.bounding.clone();
                rect.height = bar_extent;
                let resolved = resolved.unwrap();
                match resolved.deref().clone() {
                    Geometry::Shape(s) => {
                        let shape = get_shape!(res, s).ok();
                        if let Some(shape) = shape {
                            Self::draw_track_shape(shape.clone(), elem, ctx, context, crop_area, &rect);
                        }
                    }
                    Geometry::Adaptive(a) => {
                        let shape = get_adaptive!(res, a).ok();
                        if let Some(shape) = shape {
                            Self::draw_track_adaptive(shape, elem, ctx, context, crop_area, &rect);
                        }
                    }
                }
            }

            let knob = Self::x_knob(state, bar_extent);

            let resolved = resolve!(elem, scrollbar.knob.shape);
            let resource = resolve!(elem, scrollbar.knob.resource);
            if resolved.is_set() && !resource.is_none() {
                let resolved = resolved.unwrap();
                match resolved.deref().clone() {
                    Geometry::Shape(s) => {
                        let shape = get_shape!(res, s).ok();
                        if let Some(shape) = shape {
                            Self::draw_knob_shape(shape.clone(), elem, ctx, context, crop_area, &knob);
                        }
                    }
                    Geometry::Adaptive(a) => {
                        let shape = get_adaptive!(res, a).ok();
                        if let Some(shape) = shape {
                            Self::draw_knob_adaptive(shape, elem, ctx, context, crop_area, &knob);
                        }
                    }
                }
            }
        }

        if state.scroll_y.available {
            let resolved = resolve!(elem, scrollbar.track.shape);
            let resource = resolve!(elem, scrollbar.track.resource);
            if resolved.is_set() && !resource.is_none() {
                let rect = SimpleRect::new(state.content_rect.x() + state.content_rect.width() - bar_extent,
                                               state.content_rect.y(),
                                               bar_extent,
                                               state.content_rect.height());
                
                let resolved = resolved.unwrap();
                match resolved.deref().clone() {
                    Geometry::Shape(s) => {
                        let shape = get_shape!(res, s).ok();
                        if let Some(shape) = shape {
                            Self::draw_track_shape(shape.clone(), elem, ctx, context, crop_area, &rect);
                        }
                    }
                    Geometry::Adaptive(a) => {
                        let shape = get_adaptive!(res, a).ok();
                        if let Some(shape) = shape {
                            Self::draw_track_adaptive(shape, elem, ctx, context, crop_area, &rect);
                        }
                    }
                }
            }

            let knob = Self::y_knob(state, bar_extent);

            let resolved = resolve!(elem, scrollbar.knob.shape);
            let resource = resolve!(elem, scrollbar.knob.resource);
            if resolved.is_set() && !resource.is_none() {
                let resolved = resolved.unwrap();
                match resolved.deref().clone() {
                    Geometry::Shape(s) => {
                        let shape = get_shape!(res, s).ok();
                        if let Some(shape) = shape {
                            Self::draw_knob_shape(shape.clone(), elem, ctx, context, crop_area, &knob);
                        }
                    }
                    Geometry::Adaptive(a) => {
                        let shape = get_adaptive!(res, a).ok();
                        if let Some(shape) = shape {
                            Self::draw_knob_adaptive(shape, elem, ctx, context, crop_area, &knob);
                        }
                    }
                }
            }
        }
    }

    pub fn x_knob(state: &UiElementState, bar_extent: i32) -> SimpleRect {        
        let knob_width = (state.content_rect.width() as f32 / state.scroll_x.whole as f32)
            * state.content_rect.width() as f32;
        let knob_width = knob_width as i32;
        SimpleRect::new(
            state.content_rect.x() + state.scroll_x.offset,
            state.content_rect.y(),
            knob_width,
            bar_extent,
        )
    }

    pub fn y_knob(state: &UiElementState, bar_extent: i32) -> SimpleRect {        
        let knob_height = (state.content_rect.height() as f32 / state.scroll_y.whole as f32)
            * state.content_rect.height() as f32;
        let knob_height = knob_height as i32;
        SimpleRect::new(
            state.content_rect.x() + state.content_rect.width() - bar_extent,
            state.content_rect.y() + state.content_rect.height() - state.scroll_y.offset - knob_height,
            bar_extent,
            knob_height,
        )
    }

    fn draw_track_shape<E: UiElementStub>(
        mut background_shape: Shape,
        elem: &E,
        ctx: &mut DrawContext2D,
        context: &UiContext,
        crop_area: &SimpleRect,
        area: &SimpleRect,
    ) {
        let state = elem.state();
        let style = elem.style();
        let bounds = area;

        background_shape.invalidate();
        let (bgsw, bgsh) = background_shape.extent;
        let bg_scale_x = bounds.width as f32 / bgsw as f32;
        let bg_scale_y = bounds.height as f32 / bgsh as f32;
        let tmp = resolve!(elem, scrollbar.track.resource);
        if !tmp.is_set() {
            return;
        }
        let bg_res = tmp.unwrap();
        let bg_res = bg_res.deref();
        let mut bg_empty = false;
        match bg_res {
            BackgroundRes::Color => {
                let color = resolve!(elem, scrollbar.track.color);
                if color.is_set() {
                    background_shape.set_color(color.unwrap());
                } else {
                    bg_empty = true;
                }
            }
            BackgroundRes::Texture => {
                if !style.scrollbar.track.texture.is_set() {
                    bg_empty = true;
                } else {
                    let tex = resolve!(elem, scrollbar.track.texture);
                    if let ResolveResult::Value(tex) = tex {
                        let tex = tex.deref().clone();
                        if let Some((tex, uv)) = tex.get_texture(context.resources) {
                            background_shape
                                .set_texture(ctx::texture().source(Some(tex.clone())).uv(uv));
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
            background_shape.set_translate(area.x, area.y);
            background_shape.apply_transformations();
            background_shape.set_origin(area.x, area.y);
            background_shape.set_scale(bg_scale_x, bg_scale_y);
            background_shape.apply_transformations();
            let ui_transform = state.inner_transforms.as_render_transform(state);
            background_shape.set_transform(ui_transform);
            background_shape.crop_to(crop_area);
            ctx.shape(background_shape);
        }
    }

    fn draw_track_adaptive<E: UiElementStub>(
        bg_shape: &AdaptiveShape,
        elem: &E,
        ctx: &mut DrawContext2D,
        context: &UiContext,
        crop_area: &SimpleRect,
        area: &SimpleRect
    ) {
        let res = resolve!(elem, scrollbar.track.resource).unwrap_or(BackgroundRes::Color.into());
        let res = res.deref();
        let fill = match res {
            BackgroundRes::Color => {
                let color = resolve!(elem, scrollbar.track.color).unwrap_or(RgbColor::white());
                AdaptiveFill::Color(color)
            }
            BackgroundRes::Texture => {
                let texture =
                    resolve!(elem, scrollbar.track.texture).unwrap_or(Drawable::missing().into());
                let texture = texture.deref().clone();
                AdaptiveFill::Drawable(texture)
            }
        };
        let transform = elem
            .state()
            .inner_transforms
            .as_render_transform(elem.state());
        let mut area = Rect::simple(area.x, area.y, area.width, area.height);
        area.transform(&transform);
        bg_shape.draw(ctx, &area, fill, context, crop_area);
    }

    fn draw_knob_shape<E: UiElementStub>(
        mut background_shape: Shape,
        elem: &E,
        ctx: &mut DrawContext2D,
        context: &UiContext,
        crop_area: &SimpleRect,
        area: &SimpleRect,
    ) {
        let state = elem.state();
        let style = elem.style();
        let bounds = area;

        background_shape.invalidate();
        let (bgsw, bgsh) = background_shape.extent;
        let bg_scale_x = bounds.width as f32 / bgsw as f32;
        let bg_scale_y = bounds.height as f32 / bgsh as f32;
        let tmp = resolve!(elem, scrollbar.knob.resource);
        if !tmp.is_set() {
            return;
        }
        let bg_res = tmp.unwrap();
        let bg_res = bg_res.deref();
        let mut bg_empty = false;
        match bg_res {
            BackgroundRes::Color => {
                let color = resolve!(elem, scrollbar.knob.color);
                if color.is_set() {
                    background_shape.set_color(color.unwrap());
                } else {
                    bg_empty = true;
                }
            }
            BackgroundRes::Texture => {
                if !style.scrollbar.track.texture.is_set() {
                    bg_empty = true;
                } else {
                    let tex = resolve!(elem, scrollbar.knob.texture);
                    if let ResolveResult::Value(tex) = tex {
                        let tex = tex.deref().clone();
                        if let Some((tex, uv)) = tex.get_texture(context.resources) {
                            background_shape
                                .set_texture(ctx::texture().source(Some(tex.clone())).uv(uv));
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
            background_shape.set_translate(area.x, area.y);
            background_shape.apply_transformations();
            background_shape.set_origin(area.x, area.y);
            background_shape.set_scale(bg_scale_x, bg_scale_y);
            background_shape.apply_transformations();
            let ui_transform = state.inner_transforms.as_render_transform(state);
            background_shape.set_transform(ui_transform);
            background_shape.crop_to(crop_area);
            ctx.shape(background_shape);
        }
    }

    fn draw_knob_adaptive<E: UiElementStub>(
        bg_shape: &AdaptiveShape,
        elem: &E,
        ctx: &mut DrawContext2D,
        context: &UiContext,
        crop_area: &SimpleRect,
        area: &SimpleRect
    ) {
        let res = resolve!(elem, scrollbar.knob.resource).unwrap_or(BackgroundRes::Color.into());
        let res = res.deref();
        let fill = match res {
            BackgroundRes::Color => {
                let color = resolve!(elem, scrollbar.knob.color).unwrap_or(RgbColor::white());
                AdaptiveFill::Color(color)
            }
            BackgroundRes::Texture => {
                let texture =
                    resolve!(elem, scrollbar.knob.texture).unwrap_or(Drawable::missing().into());
                let texture = texture.deref().clone();
                AdaptiveFill::Drawable(texture)
            }
        };
        let transform = elem
            .state()
            .inner_transforms
            .as_render_transform(elem.state());
        let mut area = Rect::simple(area.x, area.y, area.width, area.height);
        area.transform(&transform);
        bg_shape.draw(ctx, &area, fill, context, crop_area);
    }
}
