use crate::rendering::text::Font;
use crate::rendering::RenderContext;
use crate::resolve;
use crate::ui::context::UiContext;
use crate::ui::elements::UiElementStub;
use crate::ui::geometry::shape::{shapes, Indices, Shape, VertexStream};
use crate::ui::geometry::SimpleRect;
use crate::ui::res::MVR;
use crate::ui::styles::ResolveResult;
use crate::ui::styles::DEFAULT_STYLE;
use std::marker::PhantomData;
use crate::color::RgbColor;
use crate::ui::styles::enums::TextAlign;

pub struct TextInfo<'a> {
    pub font : &'a Font,
    pub color: RgbColor,
    pub select_color: RgbColor,
    pub size: f32,
    pub kerning: f32,
    pub stretch_x: f32,
    pub stretch_y: f32,
    pub skew: f32,
    pub align_x: TextAlign,
    pub align_y: TextAlign,
    pub space_adv: f32,
    pub max_y_off: f32,
}

#[derive(Clone)]
pub struct BoringText<E: UiElementStub> {
    _phantom: PhantomData<E>,
}

impl<E: UiElementStub> BoringText<E> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData::default(),
        }
    }

    pub fn get_info(&self, elem: &E, context: &UiContext) -> Option<TextInfo> {
        let text_align_x =
            resolve!(elem, text.align_x).unwrap_or_default(&DEFAULT_STYLE.text.align_x);
        let text_align_y =
            resolve!(elem, text.align_y).unwrap_or_default(&DEFAULT_STYLE.text.align_y);
        let font = resolve!(elem, text.font);
        let font = font.unwrap_or(MVR.font.default);
        if let Some(font) = context.resources.resolve_font(font) {
            let color = resolve!(elem, text.color).unwrap_or_default(&DEFAULT_STYLE.text.color);
            let select_color = resolve!(elem, text.select_color).unwrap_or_default(&DEFAULT_STYLE.text.select_color);
            let size = resolve!(elem, text.size).unwrap_or_default(&DEFAULT_STYLE.text.size);
            let kerning =
                resolve!(elem, text.kerning).unwrap_or_default(&DEFAULT_STYLE.text.kerning);
            let stretch =
                resolve!(elem, text.stretch).unwrap_or_default(&DEFAULT_STYLE.text.stretch);
            let skew = resolve!(elem, text.skew).unwrap_or_default(&DEFAULT_STYLE.text.skew);

            let ssize = size * stretch.height;

            let max_y_off = font.get_max_y_off(ssize);
            let space_adv = font.get_space_advance(ssize);

            Some(TextInfo {
                font,
                color,
                select_color,
                size: ssize,
                kerning,
                stretch_x: stretch.width,
                stretch_y: stretch.height,
                skew,
                align_x: text_align_x,
                align_y: text_align_y,
                space_adv,
                max_y_off,
            })
        } else {
            None
        }
    }

    pub fn draw(
        &self,
        text: &str,
        elem: &E,
        ctx: &mut impl RenderContext,
        context: &UiContext,
        crop: &SimpleRect,
    ) {
        if let Some(info) = self.get_info(elem, context) {
            self.draw_with_info(text, elem, ctx, crop, info);
        }
    }

    pub fn draw_with_info(
        &self,
        text: &str,
        elem: &E,
        ctx: &mut impl RenderContext,
        crop: &SimpleRect,
        info: TextInfo,
    ) {
        let state = elem.state();

        let mut x = state.content_rect.x() as f32;
        let y = state.content_rect.y() as f32;

        for c in text.chars() {
            let cwidth = self.draw_char(c, &info, x, y, ctx, crop);

            x += cwidth + info.kerning;
        }
    }

    pub fn draw_char(&self, c: char, info: &TextInfo, x: f32, y: f32, ctx: &mut impl RenderContext, crop: &SimpleRect) -> f32 {
        if c == ' ' {
            return info.space_adv;
        } else if c == '\t' {
            return info.space_adv * 4.0;
        }

        let data = info.font.get_char_data(c, info.size);
        let ssize = data.size;
        let cwidth = data.width * info.stretch_x;

        let y = y + data.y_off;

        let bl = shapes::vertex3(
            x - info.skew,
            y,
            info.font.texture().id,
            (data.uv.x, 1.0 - (data.uv.y + data.uv.w)),
        );
        let tl = shapes::vertex3(
            x + info.skew,
            y + ssize,
            info.font.texture().id,
            (data.uv.x, 1.0 - data.uv.y),
        );
        let tr = shapes::vertex3(
            x + info.skew + cwidth,
            y + ssize,
            info.font.texture().id,
            (data.uv.x + data.uv.z, 1.0 - data.uv.y),
        );
        let br = shapes::vertex3(
            x - info.skew + cwidth,
            y,
            info.font.texture().id,
            (data.uv.x + data.uv.z, 1.0 - (data.uv.y + data.uv.w)),
        );

        let mut shape = Shape::new(vec![tl, bl, tr, br], Indices::TriangleStrip);
        shape.recompute();
        let area = &shape.extent;
        shape.draw(ctx, |v| {
            v.has_texture = 2.0;
            v.color = info.color.as_vec4();
            v.pos.0 = v.pos.0.clamp(crop.x as f32, (crop.x + crop.width) as f32);
            v.pos.1 = v.pos.1.clamp(crop.y as f32, (crop.y + crop.height) as f32);

            //custom uv crop code cuz the uv for text is weird
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

                let uv_x1 = data.uv.x;
                let uv_x2 = data.uv.x + data.uv.z;
                let uv_y1 = 1.0 - (data.uv.y + data.uv.w);
                let uv_y2 = 1.0 - data.uv.y;

                v.uv.0 = uv_x1 + x_ratio * (uv_x2 - uv_x1);
                v.uv.1 = uv_y1 + y_ratio * (uv_y2 - uv_y1);
            }
        });

        cwidth
    }
}
