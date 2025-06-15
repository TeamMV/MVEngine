use crate::rendering::{InputVertex, Quad, Transform};
use crate::rendering::text::Font;
use crate::ui::styles::ResolveResult;
use crate::resolve;
use crate::ui::elements::UiElementStub;
use crate::ui::geometry::shape::Shape;
use crate::ui::geometry::SimpleRect;
use crate::ui::rendering::ctx::DrawContext2D;
use crate::ui::res::MVR;
use crate::ui::styles::{InheritSupplier, DEFAULT_STYLE};

#[derive(Clone)]
pub struct TextBody {

}

impl TextBody {
    pub fn draw<E: UiElementStub + 'static>(&self, elem: &mut E, s: &str, ctx: &mut DrawContext2D, crop_area: &SimpleRect) {
        let state = elem.state();

        let text_size = resolve!(elem, text.size).unwrap_or_default_or_percentage(&DEFAULT_STYLE.text.size, state.parent.clone(), |s| s.height() as f32, state);
        let font = resolve!(elem, text.font);
        let font = font.unwrap_or(MVR.font.default);
        if let Some(font) = elem.context().resources.resolve_font(font) {
            let color = resolve!(elem, text.color).unwrap_or_default(&DEFAULT_STYLE.text.color);
            let kerning =
                resolve!(elem, text.kerning).unwrap_or_default(&DEFAULT_STYLE.text.kerning);
            let stretch =
                resolve!(elem, text.stretch).unwrap_or_default(&DEFAULT_STYLE.text.stretch);
            let skew = resolve!(elem, text.skew).unwrap_or_default(&DEFAULT_STYLE.text.skew);

            let mut y = 0f32;

            let lines = self.split_up(state.content_rect.width(), text_size, s, font);
            for line in lines.iter().rev() {
                let mut x = 0f32;
                let space_advance = font.get_space_advance(text_size);
                let mut height = 0;
                for c in line.chars() {
                    if c == '\t' {
                        x += 6.0 + space_advance;
                        continue;
                    } else if c == ' ' {
                        x += space_advance;
                        continue;
                    } else if c == '\n' {
                        continue;
                    }
                    let data = font.get_char_data(c, text_size);
                    let vertex = InputVertex {
                        transform: Transform::new(),
                        pos: (x, y, f32::INFINITY),
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

                    let mut shape = Shape::new(quad.triangles().to_vec());
                    shape.is_quad = true;
                    let mut shape = shape.translated(state.content_rect.x(), state.content_rect.y());
                    shape.apply_transformations();
                    shape.crop_to(crop_area);
                    ctx.shape(shape);

                    x += data.width * stretch.width + kerning + skew * 2f32;
                }

                y += text_size + 2f32;
            }
            
            elem.state_mut().requested_height = Some(y as i32);
        }
    }

    fn split_up(&self, width: i32, size: f32, s: &str, font: &Font) -> Vec<String> {
        let mut result = Vec::new();
        let mut line_start = 0;
        let mut last_space = None;
        let mut current_width = 0.0;

        let mut i = 0;
        let chars: Vec<char> = s.chars().collect();

        while i < chars.len() {
            let c = chars[i];

            if c == ' ' {
                last_space = Some(i);
            }

            let substr: String = chars[line_start..=i].iter().collect();
            current_width = font.get_width(&substr, size);

            if current_width > width as f32 {
                if let Some(space_idx) = last_space {
                    let line: &str = &s[line_start..space_idx];
                    result.push(line.trim_end().to_string());

                    i = space_idx + 1; // skip the space
                    line_start = i;
                    last_space = None;
                    continue;
                } else {
                    // No space found, break at current position
                    let line: &str = &s[line_start..i];
                    result.push(line.trim_end().to_string());

                    line_start = i;
                }
            }

            i += 1;
        }

        if line_start < chars.len() {
            let line: &str = &s[line_start..];
            result.push(line.trim_end().to_string());
        }

        result
    }
}