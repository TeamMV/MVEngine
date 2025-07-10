use std::ops::Range;
use mvutils::utils::TetrahedronOp;
use crate::color::RgbColor;
use crate::rendering::RenderContext;
use crate::ui::attributes::UiState;
use crate::ui::context::UiContext;
use crate::ui::elements::components::boring::BoringText;
use crate::ui::elements::UiElementStub;
use crate::ui::geometry::shape::shapes;
use crate::ui::geometry::{shape, SimpleRect};
use crate::ui::rendering::WideRenderContext;
use ropey::Rope;
use crate::ui::styles::enums::TextAlign;
use crate::utils::RopeFns;

#[derive(Clone)]
enum Cursor {
    Single(usize),
    Range {
        start: usize,
        end: usize,
        on_start: bool
    }
}

impl Cursor {
    fn get_cursor_pos(&self) -> usize {
        match self {
            Cursor::Single(p) => *p,
            Cursor::Range { start, end, on_start } => on_start.yn(*start, *end),
        }
    }

    fn contains(&self, i: usize) -> bool {
        match self {
            Cursor::Single(_) => false,
            Cursor::Range { start, end, .. } => i >= *start && i < *end,
        }
    }

    fn maybe_move(&self) -> usize {
        match self {
            Cursor::Single(_) => 1,
            Cursor::Range { .. } => 0
        }
    }
}

#[derive(Clone)]
pub struct EditableTextHelper<E: UiElementStub> {
    content: UiState,
    text_body: BoringText<E>,
    cursor: Cursor,
    view_range: Range<usize>,
}

impl<E: UiElementStub> EditableTextHelper<E> {
    pub fn new(content: UiState) -> Self {
        Self {
            content,
            text_body: BoringText::new(),
            cursor: Cursor::Single(0),
            view_range: Range::default(),
        }
    }

    pub fn move_left(&mut self, select: bool) {
        if select {
            match self.cursor {
                Cursor::Single(end) => {
                    if end != 0 {
                        self.cursor = Cursor::Range {
                            start: end - 1,
                            end,
                            on_start: true,
                        }
                    }
                }
                Cursor::Range {
                    start, end, on_start
                } => {
                    if on_start {
                        self.cursor = Cursor::Range {
                            start: start.saturating_sub(1),
                            end,
                            on_start,
                        }
                    } else {
                        let end = end.saturating_sub(1);
                        if end <= start {
                            self.cursor = Cursor::Single(end);
                        } else {
                            self.cursor = Cursor::Range {
                                start,
                                end,
                                on_start,
                            }
                        }
                    }
                }
            }
        } else {
            self.cursor = Cursor::Single(self.cursor.get_cursor_pos().saturating_sub(self.cursor.maybe_move()));
        }
    }

    pub fn move_right(&mut self, select: bool) {
        let len = self.content.read().len_chars();
        if select {
            match self.cursor {
                Cursor::Single(start) => {
                    if start != len {
                        self.cursor = Cursor::Range {
                            start,
                            end: start + 1,
                            on_start: false,
                        }
                    }
                }
                Cursor::Range {
                    start, end, on_start
                } => {
                    if !on_start {
                        let end = if end == len { end } else { end + 1 };
                        self.cursor = Cursor::Range {
                            start,
                            end,
                            on_start,
                        }
                    } else {
                        let start = start + 1;
                        if end <= start {
                            self.cursor = Cursor::Single(start);
                        } else {
                            self.cursor = Cursor::Range {
                                start,
                                end,
                                on_start,
                            }
                        }
                    }
                }
            }
        } else {
            let mut pos = self.cursor.get_cursor_pos();
            if pos < len { pos += self.cursor.maybe_move(); }
            self.cursor = Cursor::Single(pos);
        }
    }

    pub fn move_to_end(&mut self, select: bool) {
        let len = self.content.read().len_chars();
        if select {
            match self.cursor {
                Cursor::Single(p) => self.cursor = Cursor::Range {
                    start: p,
                    end: len,
                    on_start: false,
                },
                Cursor::Range { start, end, on_start } => {
                    self.cursor = if on_start {
                        Cursor::Range {
                            start: end,
                            end: len,
                            on_start: false
                        }
                    } else {
                        Cursor::Range {
                            start,
                            end: len,
                            on_start: false
                        }
                    };
                }
            }
        } else {
            self.cursor = Cursor::Single(len);
        }
    }

    pub fn move_to_start(&mut self, select: bool) {
        if select {
            match self.cursor {
                Cursor::Single(p) => self.cursor = Cursor::Range {
                    start: 0,
                    end: p,
                    on_start: true,
                },
                Cursor::Range { start, end, on_start } => {
                    self.cursor = if on_start {
                        Cursor::Range {
                            start: 0,
                            end,
                            on_start: true
                        }
                    } else {
                        Cursor::Range {
                            start: 0,
                            end: start,
                            on_start: true
                        }
                    };
                }
            }
        } else {
            self.cursor = Cursor::Single(0);
        }
    }

    pub fn add_str(&mut self, s: &str) {
        let r = Rope::from_str(s);
        let len = r.len_chars();
        let mut g = self.content.write();
        match self.cursor {
            Cursor::Single(p) => {
                g.insert(p, s);
                self.cursor = Cursor::Single(p + len);
                self.view_range.end += len;
            }
            Cursor::Range { start, end, on_start } => {
                g.remove(start..end);
                g.insert(start, s);
                self.cursor = Cursor::Single(start + len);
                self.view_range.end = self.view_range.end.saturating_sub(end - start);
                self.view_range.end += len;
            }
        }
    }

    pub fn backspace(&mut self) {
        let mut g = self.content.write();
        match self.cursor {
            Cursor::Single(p) => {
                if p > 0 {
                    g.remove_char(p - 1);
                    self.cursor = Cursor::Single(p - 1);
                    if self.view_range.start > 0 {
                        self.view_range.start -= 1;
                    }
                    if self.view_range.end > 0 {
                        self.view_range.end -= 1;
                    }
                }
            }
            Cursor::Range { mut start, end, .. } => {
                g.remove(start..end);
                self.cursor = Cursor::Single(start);
                let deleted_len = end - start;
                self.view_range.end = self.view_range.end.saturating_sub(deleted_len);
                self.view_range.start = self.view_range.start.min(start);
            }
        }
    }

    pub fn draw(&mut self, elem: &E, draw_ctx: &mut impl WideRenderContext, ui_ctx: &UiContext, crop: &SimpleRect, draw_cursor: bool) {
        let s = self.content.read();
        let cursor_pos = self.cursor.get_cursor_pos();
        let diff = self.view_range.start as isize - cursor_pos as isize;
        if diff > 0 {
            self.view_range.start = self.view_range.start.saturating_sub(diff as usize);
            self.view_range.end = self.view_range.end.saturating_sub(diff as usize);
        }

        let rect = &elem.state().content_rect;
        let max_x = rect.width() + rect.x();

        let info = self.text_body.get_info(elem, ui_ctx, draw_ctx);
        if let Some(mut info) = info {
            let m_width = info.font.get_char_data('m', info.size).width;
            println!("s: {}, e: {}", self.view_range.start, self.view_range.end);
            if self.view_range.start == self.view_range.end {
                self.view_range.start = self.view_range.start.saturating_sub((rect.width() / m_width as i32) as usize);
            }

            let y = rect.y() as f32;
            let height = rect.height() as f32;
            let mut char_x = rect.x() as f32;
            //println!("align_y: {:?}", info.align_y);
            let char_y = match info.align_y {
                TextAlign::Start => y,
                TextAlign::Middle => y + (height - info.size) * 0.5,
                TextAlign::End => y + height - info.size
            };

            let mut cursor_found = false;
            if (s.is_empty() || cursor_pos == self.view_range.start) && draw_cursor {
                Self::draw_cursor(draw_ctx, char_x, char_y, info.size, &info.color, crop);
                cursor_found = true;
            }

            let mut sel_rect_start = None;
            let mut sel_rect_end = None;
            let cache = info.color.clone();

            let sel_z = draw_ctx.next_z();

            for (i, c) in s.chars().enumerate().skip(self.view_range.start) {
                if self.cursor.contains(i) {
                    if sel_rect_start.is_none() {
                        sel_rect_start = Some(char_x);
                        info.color = RgbColor::white();
                    }
                } else {
                    if sel_rect_start.is_some() && sel_rect_end.is_none() {
                        //this is next char after last char in sel
                        sel_rect_end = Some(char_x);
                        info.color = cache.clone();
                    }
                }
                let char_width = self.text_body.draw_char(c, &info, char_x, char_y, draw_ctx, crop);

                char_x += char_width;

                if i + 1 == cursor_pos {
                    if draw_cursor {
                        Self::draw_cursor(draw_ctx, char_x, char_y, info.size, &info.color, crop);
                    }
                    cursor_found = true;
                }

                if char_x > max_x as f32 {
                    self.view_range.end = i;
                    if !cursor_found && cursor_pos != 0 {
                        if draw_cursor {
                            Self::draw_cursor(draw_ctx, char_x - char_width, char_y, info.size, &info.color, crop);
                        }
                        let amount = cursor_pos - i;
                        self.view_range.start += amount;
                        self.view_range.end += amount;
                    }
                }
            }

            if let Some(start_x) = sel_rect_start {
                let end_x = if let Some(end_x) = sel_rect_end { end_x } else { char_x };
                let rect = shapes::rectangle1(start_x as i32, char_y as i32, end_x as i32, (char_y + info.size) as i32);
                rect.draw(draw_ctx, |v| {
                    v.color = info.select_color.as_vec4();
                    v.pos.2 = sel_z;
                })
            }
        }
    }

    fn draw_cursor(ctx: &mut impl RenderContext, x: f32, y: f32, height: f32, col: &RgbColor, crop: &SimpleRect) {
        let cursor_rect = shapes::rectangle0(x as i32, y as i32, 2, height as i32);
        cursor_rect.draw(ctx, |v| {
            shape::utils::crop_no_uv(v, crop);
            v.color = col.as_vec4();
        });
    }

    pub fn draw_other(&mut self, s: &Rope, elem: &E, draw_ctx: &mut impl WideRenderContext, ui_ctx: &UiContext, crop: &SimpleRect) {
        self.text_body.draw(0, 0, s, elem, draw_ctx, ui_ctx, crop);
    }
}
