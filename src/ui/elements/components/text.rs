use std::marker::PhantomData;
use crate::color::RgbColor;
use crate::rendering::{InputVertex, Quad, Transform};
use crate::rendering::text::Font;
use crate::resolve;
use crate::ui::styles::ResolveResult;
use crate::ui::context::{UiContext, UiResources};
use crate::ui::elements::UiElementStub;
use crate::ui::geometry::shape::Shape;
use crate::ui::rendering::ctx::DrawContext2D;
use crate::ui::res::MVR;
use crate::ui::styles::DEFAULT_STYLE;
use crate::ui::styles::enums::TextAlign;
use crate::ui::styles::types::Dimension;

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