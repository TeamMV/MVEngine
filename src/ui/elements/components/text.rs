use crate::rendering::text::Font;
use crate::rendering::RenderContext;
use crate::ui::elements::UiElementStub;
use crate::ui::geometry::SimpleRect;

#[derive(Clone)]
pub struct TextBody {

}

impl TextBody {
    pub fn draw<E: UiElementStub + 'static>(&self, elem: &mut E, s: &str, ctx: &mut impl RenderContext, crop_area: &SimpleRect) {
        //TODO
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