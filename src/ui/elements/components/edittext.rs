use std::mem;
use crate::color::RgbColor;
use crate::rendering::RenderContext;
use crate::ui::attributes::UiState;
use crate::ui::context::UiContext;
use crate::ui::elements::components::boring::BoringText;
use crate::ui::elements::UiElementStub;
use crate::ui::geometry::shape::shapes;
use crate::ui::geometry::{shape, SimpleRect};
use itertools::Itertools;
use std::ops::Range;
use ropey::Rope;
use crate::ui::rendering::WideRenderContext;
use crate::ui::styles::enums::TextAlign;
use crate::utils::RopeFns;

#[derive(Clone)]
pub struct EditableTextHelper<E: UiElementStub> {
    cursor_pos: usize,
    selection: Option<Range<usize>>,
    content: UiState,
    text_body: BoringText<E>,
    pub(crate) view_range: Range<usize>,
}

impl<E: UiElementStub> EditableTextHelper<E> {
    pub fn new(content: UiState) -> Self {
        let l = content.read().len_chars();
        let view_range = 0..l;
        Self {
            cursor_pos: 0,
            selection: None,
            content,
            text_body: BoringText::new(),
            view_range,
        }
    }

    pub fn move_left(&mut self, select: bool) {
        if self.cursor_pos == 0 {
            return;
        }

        if select {
            if let Some(range) = self.selection.clone() {
                if self.cursor_pos >= range.end {
                    self.selection = Some(range.start..(range.end - 1));
                } else {
                    self.selection = Some((range.start - 1)..range.end);
                }
            } else {
                self.selection = Some((self.cursor_pos - 1)..self.cursor_pos);
            }
        } else {
            self.selection = None;
        }

        self.cursor_pos -= 1;
        if self.cursor_pos < self.view_range.start {
            self.view_range.start -= 1;
        }
    }

    pub fn move_right(&mut self, select: bool) {
        if self.cursor_pos >= self.content.read().len_chars() {
            return;
        }

        if select {
            if let Some(range) = self.selection.clone() {
                if self.cursor_pos < range.end {
                    self.selection = Some((range.start + 1)..range.end);
                } else {
                    self.selection = Some(range.start..(range.end + 1));
                }
            } else {
                self.selection = Some(self.cursor_pos..(self.cursor_pos + 1));
            }
        } else {
            self.selection = None;
        }

        self.cursor_pos += 1;
    }

    pub fn move_to_end(&mut self, select: bool) {
        let to_move = self.content.read().len_chars() - self.cursor_pos;
        if select {
            if let Some(range) = self.selection.clone() {
                if self.cursor_pos < range.end {
                    self.selection = Some((range.start + to_move)..range.end);
                } else {
                    self.selection = Some(range.start..(range.end + to_move));
                }
            } else {
                self.selection = Some(self.cursor_pos..(self.cursor_pos + to_move));
            }
        } else {
            self.selection = None;
        }

        self.cursor_pos += to_move;
    }

    pub fn move_to_start(&mut self, select: bool) {
        let to_move = self.cursor_pos;

        if select {
            if let Some(range) = self.selection.clone() {
                if self.cursor_pos >= range.end {
                    self.selection = Some(range.start..(range.end - to_move));
                } else {
                    self.selection = Some((range.start - to_move)..range.end);
                }
            } else {
                self.selection = Some((self.cursor_pos - to_move)..self.cursor_pos);
            }
        } else {
            self.selection = None;
        }

        self.cursor_pos -= to_move;
        if self.cursor_pos < self.view_range.start {
            self.view_range.start = 0;
        }
    }

    pub fn add_str(&mut self, s: &str) {
        let mut guard = self.content.write();

        let rope = Rope::from_str(s);

        if let Some(range) = self.selection.take() {
            let start = range.start.min(guard.len_chars());
            let end = range.end.min(guard.len_chars());

            guard.replace_range(start..end, "");
            guard.insert_str(start, s);

            self.cursor_pos = start + rope.len_chars();

            let replaced_len = end - start;
            let added_len = rope.len_chars();
            self.view_range.end = self.view_range.end.saturating_sub(replaced_len);
            self.view_range.end += added_len;
            self.view_range.start = self.view_range.start.min(self.cursor_pos);
        } else {
            guard.insert_str(self.cursor_pos, s);
            let l = rope.len_chars();
            self.cursor_pos += l;
            self.view_range.end += l;
        }
    }

    pub fn backspace(&mut self) {
        let mut guard = self.content.write();

        if let Some(range) = self.selection.take() {
            let start = range.start.min(guard.len_chars());
            let end = range.end.min(guard.len_chars());

            guard.replace_range(start..end, "");

            self.cursor_pos = start;

            let deleted_len = end - start;
            self.view_range.end = self.view_range.end.saturating_sub(deleted_len);
            self.view_range.start = self.view_range.start.min(self.cursor_pos);
        } else {
            if self.cursor_pos == 0 {
                return;
            }

            guard.remove_char(self.cursor_pos - 1);
            self.cursor_pos -= 1;

            self.view_range.end = self.view_range.end.saturating_sub(1);
            if self.view_range.start > 0 {
                self.view_range.start -= 1;
            }
        }
    }

    pub fn draw(&mut self, elem: &E, draw_ctx: &mut impl WideRenderContext, ui_ctx: &UiContext, crop: &SimpleRect, draw_cursor: bool) {
        let text = &*self.content.read();

        //fix selection when it is messed up
        if let Some(r) = &mut self.selection {
            if r.end < r.start {
                let tmp = r.start;
                r.start = r.end;
                r.end = tmp;
            }
        }

        if let Some(mut info) = self.text_body.get_info(elem, ui_ctx, draw_ctx) {
            let cursor_offset_rel = self.cursor_pos - self.view_range.start;

            let state = elem.state();

            let mut x = state.content_rect.x() as f32;
            let max_x = state.content_rect.width() as f32 + x;
            let y = state.content_rect.y() as f32;
            let y = match info.align_y {
                TextAlign::Start => y,
                TextAlign::Middle => y + (state.content_rect.height() as f32 - info.size) * 0.5,
                TextAlign::End => y + state.content_rect.height() as f32 - info.size,
            };
            let cursor_height = info.size as i32;
            let cursor_y = y;

            if text.is_empty() {
                if draw_cursor {
                    Self::draw_cursor(draw_ctx, x, cursor_y, cursor_height, &info.color, crop);
                }
                return;
            }

            let mut sel_start_x = None;
            let mut sel_end_x = None;
            let mut in_sel = false;

            let sel_rect_z = draw_ctx.next_z();
            let original_color = info.color.clone();

            for (i, c) in text.chars().enumerate().skip(self.view_range.start) {
                let new_x = x + self.text_body.char_width(c, &info);
                if new_x > max_x {
                    //no more characters fit
                    //if cursor is offscreen, move view range
                    let diff = cursor_offset_rel as i32 - i as i32;
                    if diff > 0 {
                        self.view_range.end += diff as usize;
                        self.view_range.start += diff as usize;
                    }
                    break;
                }

                let old_x = x;
                info.color = original_color.clone();

                let was_sel = in_sel;

                if let Some(r) = &self.selection {
                    let real_i = i + self.view_range.start;
                    if r.contains(&real_i) {
                        //this char is currently inside the selection
                        info.color = RgbColor::white();
                        if sel_start_x.is_none() {
                            sel_start_x = Some(x);
                        }
                        in_sel = true;
                    } else {
                        in_sel = false;
                    }
                } else {
                    in_sel = false;
                }

                if was_sel && !in_sel {
                    //the selection ends here
                    sel_end_x = Some(x);
                }

                x += self.text_body.draw_char(c, &info, x, y, draw_ctx, crop);

                if draw_cursor && (i + 1 == cursor_offset_rel) {
                    //draw cursor
                    Self::draw_cursor(draw_ctx, x, cursor_y, cursor_height, &info.color, crop);
                    println!("cursor index: {cursor_offset_rel}, pos: {}", self.cursor_pos);
                } else {
                    //for some reason this is a special case when the cursor is at the start lmao
                    if i == 0 && cursor_offset_rel == 0 {
                        Self::draw_cursor(draw_ctx, old_x, cursor_y, cursor_height, &info.color, crop);
                    }
                }

                if x > max_x {
                    //no more characters fit
                    //if cursor is offscreen, move view range
                    let diff = cursor_offset_rel as i32 - (i as i32 + 1);
                    if diff > 0 {
                        self.view_range.end += diff as usize;
                        self.view_range.start += diff as usize;
                    }
                    break;
                }
            }

            if sel_start_x.is_some() && sel_end_x.is_none() {
                sel_end_x = Some(x); //Bro this is the biggest tape fix ever
            }

            if let Some(sel_start_x) = sel_start_x && let Some(sel_end_x) = sel_end_x {
                let rect = shapes::rectangle1(sel_start_x as i32, cursor_y as i32, sel_end_x as i32, cursor_height + cursor_y as i32);
                rect.draw(draw_ctx, |v| {
                    shape::utils::crop_no_uv(v, crop);
                    v.color = info.select_color.as_vec4();
                    v.pos.2 = sel_rect_z;
                });
            }
        }
    }

    fn draw_cursor(ctx: &mut impl RenderContext, x: f32, y: f32, height: i32, col: &RgbColor, crop: &SimpleRect) {
        let cursor_rect = shapes::rectangle0(x as i32, y as i32, 2, height);
        cursor_rect.draw(ctx, |v| {
            shape::utils::crop_no_uv(v, crop);
            v.color = col.as_vec4();
        });
    }

    pub fn draw_other(&mut self, s: &Rope, elem: &E, draw_ctx: &mut impl WideRenderContext, ui_ctx: &UiContext, crop: &SimpleRect) {
        self.text_body.draw(0, 0, s, elem, draw_ctx, ui_ctx, crop);
    }
}
