use crate::color::RgbColor;
use crate::math::vec::Vec4;
use crate::rendering::text::Font;
use crate::rendering::texture::Texture;
use crate::rendering::{InputVertex, Quad, RenderContext, Transform};
use crate::resolve;
use crate::ui::context::UiContext;
use crate::ui::elements::UiElementStub;
use crate::ui::geometry::shape::{Indices, Shape, VertexStream, shapes};
use crate::ui::geometry::{SimpleRect, shape};
use crate::ui::res::MVR;
use crate::ui::styles::DEFAULT_STYLE;
use crate::ui::styles::ResolveResult;
use crate::ui::styles::enums::TextAlign;
use crate::ui::styles::types::Dimension;
use std::convert::identity;
use std::marker::PhantomData;

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

    pub fn draw(
        &self,
        text: &str,
        elem: &E,
        ctx: &mut impl RenderContext,
        context: &UiContext,
        crop: &SimpleRect,
    ) {
        let text_align_x =
            resolve!(elem, text.align_x).unwrap_or_default(&DEFAULT_STYLE.text.align_x);
        let text_align_y =
            resolve!(elem, text.align_y).unwrap_or_default(&DEFAULT_STYLE.text.align_y);
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

            let ssize = size * stretch.height;

            let state = elem.state();

            let mut x = state.content_rect.x() as f32;
            let y = state.content_rect.y() as f32;
            for c in text.chars() {
                let data = font.get_char_data(c, ssize);
                let ssize = data.size;
                let cwidth = data.width * stretch.width;

                let y = y + data.y_off;

                let bl = shapes::vertex3(
                    x - skew,
                    y,
                    font.texture().id,
                    (data.uv.x, 1.0 - (data.uv.y + data.uv.w)),
                );
                let tl = shapes::vertex3(
                    x + skew,
                    y + ssize,
                    font.texture().id,
                    (data.uv.x, 1.0 - data.uv.y),
                );
                let tr = shapes::vertex3(
                    x + skew + cwidth,
                    y + ssize,
                    font.texture().id,
                    (data.uv.x + data.uv.z, 1.0 - data.uv.y),
                );
                let br = shapes::vertex3(
                    x - skew + cwidth,
                    y,
                    font.texture().id,
                    (data.uv.x + data.uv.z, 1.0 - (data.uv.y + data.uv.w)),
                );

                let mut shape = Shape::new(vec![tl, bl, tr, br], Indices::TriangleStrip);
                shape.recompute();
                let area = &shape.extent;
                shape.draw(ctx, |v| {
                    v.has_texture = 2.0;
                    v.color = color.as_vec4();
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

                x += cwidth + kerning + 2.0 * skew;
            }
        }
    }
}
