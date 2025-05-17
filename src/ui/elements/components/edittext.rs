use std::marker::PhantomData;
use std::ops::Range;
use std::mem;
use crate::color::RgbColor;
use crate::resolve;
use crate::ui::attributes::UiState;
use crate::ui::styles::ResolveResult;
use crate::ui::context::{UiContext, UiResources};
use crate::ui::elements::UiElementStub;
use crate::ui::rendering::ctx;
use crate::ui::rendering::ctx::DrawContext2D;
use crate::ui::res::MVR;
use crate::ui::styles::DEFAULT_STYLE;
use crate::ui::styles::enums::TextAlign;

#[derive(Clone)]
pub struct EditableTextHelper<E: UiElementStub> {
    _phantom: PhantomData<E>,
    cursor_pos: usize,
    selection: Option<Range<usize>>,
    content: UiState,
    pub(crate) view_range: Range<usize>
}

impl<E: UiElementStub> EditableTextHelper<E> {
    pub fn new(content: UiState) -> Self {
        let l = content.read().len();
        let view_range = 0..l;
        Self {
            _phantom: PhantomData::default(),
            cursor_pos: 0,
            selection: None,
            content,
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
        if self.cursor_pos >= self.content.read().len() {
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
        let to_move = self.content.read().len() - self.cursor_pos;
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

        if let Some(range) = self.selection.take() {
            let start = range.start.min(guard.len());
            let end = range.end.min(guard.len());
            
            guard.replace_range(start..end, "");
            guard.insert_str(start, s);
            
            self.cursor_pos = start + s.len();
            
            let replaced_len = end - start;
            let added_len = s.len();
            self.view_range.end = self.view_range.end.saturating_sub(replaced_len);
            self.view_range.end += added_len;
            self.view_range.start = self.view_range.start.min(self.cursor_pos);
        } else {
            guard.insert_str(self.cursor_pos, s);
            self.cursor_pos += s.len();
            self.view_range.end += s.len();
        }
    }

    pub fn backspace(&mut self) {
        let mut guard = self.content.write();
        
        if let Some(range) = self.selection.take() {
            let start = range.start.min(guard.len());
            let end = range.end.min(guard.len());
            
            guard.replace_range(start..end, "");
            
            self.cursor_pos = start;
            
            let deleted_len = end - start;
            self.view_range.end = self.view_range.end.saturating_sub(deleted_len);
            self.view_range.start = self.view_range.start.min(self.cursor_pos);
        } else {
            if self.cursor_pos == 0 { return; }

            guard.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;

            self.view_range.end = self.view_range.end.saturating_sub(1);
            if self.view_range.start > 0 {
                self.view_range.start -= 1;
            }
        }
    }

    pub fn draw(&mut self, elem: &E, draw_ctx: &mut DrawContext2D, ui_ctx: &UiContext) {
        let text_align_x = resolve!(elem, text.align_x).unwrap_or(TextAlign::Middle);
        let text_align_y = resolve!(elem, text.align_y).unwrap_or(TextAlign::Middle);
        let font = resolve!(elem, text.font).unwrap_or(MVR.font.default);

        if let Some(font) = ui_ctx.resources.resolve_font(font) {
            let color = resolve!(elem, text.color).unwrap_or_default(&DEFAULT_STYLE.text.color);
            let size = resolve!(elem, text.size).unwrap_or_default(&DEFAULT_STYLE.text.size);
            let kerning = resolve!(elem, text.kerning).unwrap_or_default(&DEFAULT_STYLE.text.kerning);
            let stretch = resolve!(elem, text.stretch).unwrap_or_default(&DEFAULT_STYLE.text.stretch);
            let skew = resolve!(elem, text.skew).unwrap_or_default(&DEFAULT_STYLE.text.skew);

            let state = elem.state();
            let font_size = size * stretch.height;
            let text_height = font_size as i32;

            let content_width = elem.state().content_rect.bounding.width;

            if self.view_range.start > self.view_range.end { 
                mem::swap(&mut self.view_range.start, &mut self.view_range.end);
            }

            let viewed_string = self.content.read();
            let viewed_string = &viewed_string[self.view_range.clone()];
            let viewed_len = viewed_string.len();

            let viewed_width = font.get_width(viewed_string, font_size) * stretch.width + skew * 2.0 + kerning * (viewed_len as f32 - 1.0);
            let viewed_width = viewed_width as i32;

            let content_ref = self.content.read();
            while self.view_range.end > self.view_range.start {
                let viewed_slice = &content_ref[self.view_range.start..self.view_range.end];
                let width = font.get_width(viewed_slice, font_size) * stretch.width
                    + skew * 2.0
                    + kerning * (viewed_slice.len() as f32 - 1.0);
                if width as i32 <= content_width {
                    break;
                }
                self.view_range.end -= 1;
            }

            let content_ref = self.content.read();
            while self.view_range.end < content_ref.len() {
                let new_view = &content_ref[self.view_range.start..=self.view_range.end];
                let new_width = font.get_width(new_view, font_size) * stretch.width
                    + skew * 2.0
                    + kerning * (new_view.len() as f32 - 1.0);
                if new_width as i32 > content_width {
                    break;
                }
                self.view_range.end += 1;
            }
            
            if !self.view_range.contains(&self.cursor_pos) {
                if self.cursor_pos >= self.view_range.end {
                    let diff = self.cursor_pos - self.view_range.end;
                    self.view_range.end += diff;
                    self.view_range.start += diff;
                } else if self.cursor_pos < self.view_range.start {
                    let diff =  self.view_range.start - self.cursor_pos + 1;
                    self.view_range.start = self.view_range.start.saturating_sub(diff);
                    self.view_range.end = self.view_range.end.saturating_sub(diff);
                }
            }
            
            self.view_range.end = self.view_range.end.min(self.content.read().len());
            
            let text_x = match text_align_x {
                TextAlign::Start => state.content_rect.x(),
                TextAlign::Middle => state.content_rect.x() + state.content_rect.width() / 2 - viewed_width / 2,
                TextAlign::End => state.content_rect.x() + state.content_rect.width() - viewed_width,
            };

            let text_y = match text_align_y {
                TextAlign::Start => state.content_rect.y(),
                TextAlign::Middle => state.content_rect.y() + state.content_rect.height() / 2 - text_height / 2,
                TextAlign::End => state.content_rect.y() + state.content_rect.height() - text_height,
            };

            let string_to_cursor = self.content.read();
            let string_to_cursor = &string_to_cursor[self.view_range.start..self.cursor_pos];
            let string_to_cursor_width = font.get_width(string_to_cursor, font_size) * stretch.width + skew * 2.0 + kerning * (string_to_cursor.len() as f32 - 1.0);
            
            let cursor_x = text_x + string_to_cursor_width as i32;
            let cursor_y = text_y;

            let cursor_width = 2.0;
            let cursor_height = text_height;
            let cursor_color = color;

            if let Some(selection) = &self.selection {
                let mut a = selection.start;
                let mut b = selection.end;
                
                if a > b { 
                    mem::swap(&mut a, &mut b);
                }
                
                let a = a.max(self.view_range.start);
                let b = b.min(self.view_range.end);
                
                let string_to_selection = self.content.read();
                let string_to_selection_a = &string_to_selection[self.view_range.start..a];
                let string_to_selection_b = &string_to_selection[self.view_range.start..b];
                let string_to_selection_a_width = font.get_width(string_to_selection_a, font_size) * stretch.width + skew * 2.0 + kerning * (string_to_selection_a.len() as f32 - 1.0);
                let string_to_selection_b_width = font.get_width(string_to_selection_b, font_size) * stretch.width + skew * 2.0 + kerning * (string_to_selection_b.len() as f32 - 1.0);
                
                let rect = ctx::rectangle()
                    .xywh(
                        text_x + string_to_selection_a_width as i32,
                        cursor_y,
                        (string_to_selection_b_width - string_to_selection_a_width) as i32,
                        cursor_height
                    )
                    .color(RgbColor::blue())
                    .create();
                draw_ctx.shape(rect);
            }
            
            let rect = ctx::rectangle()
                .xywh(cursor_x, cursor_y, cursor_width as i32, cursor_height)
                .color(cursor_color)
                .create();
            draw_ctx.shape(rect);
        }
    }
}