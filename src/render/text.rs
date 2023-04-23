use std::{cell::RefCell, rc::Rc, collections::HashMap};

use super::{shared::Texture, RenderCore};

pub struct Font {
    texture: Rc<RefCell<Texture>>,
    alphabet: Vec<Glyph>
}

impl Font {
    pub(crate) fn get_texture(&self) -> Rc<RefCell<Texture>> {
        self.texture.clone()
    }

    pub(crate) fn supports(&self, c: char) -> bool {
        false
    }

    pub(crate) fn get_glyph(&self, c: char) -> &Glyph {
        todo!()
    }

    pub(crate) fn get_max_height(&self, height: i32) -> i32 {
        0
    }

    pub fn get_metrics(&self, string: &str) -> FontMetrics {
        FontMetrics { font: self, string: string.to_string() }
    }
}


pub struct FontMetrics {
    font: *const Font,
    string: String
}

impl FontMetrics {
    pub fn fits(&self, width: i32, height: i32) -> i32 {
        0
    }

    pub fn width(&self, height: i32) -> i32 {
        let mut ret = 0;
        for c in self.string.chars() {
            ret += self.char_width(height, c);
        }
        ret
    }

    pub fn char_width(&self, height: i32, c: char) -> i32 {
        let glyph = unsafe { self.font.as_ref().unwrap().get_glyph(c) };
        glyph.get_width(height) * (height / glyph.get_height(height))
    }
}

pub struct Glyph {
    uv: [f32; 4],
    c: char,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    x_off: u16,
    y_off: u16,
    x_adv: u16
}

#[allow(clippy::too_many_arguments)]
impl Glyph {
    fn new(x: u16, y: u16, width: u16, height: u16, x_off: u16, y_off: u16, x_adv: u16, c: char) -> Self {
        Glyph { uv: [0.0; 4], c, x, y, width, height, x_off, y_off, x_adv}
    }

    fn make_coords(&mut self, atlas_width: u16, atlas_height: u16, max_height: u16) {
        self.uv[0] = self.x as f32 / atlas_width as f32;
        self.uv[1] = (self.x + self.width) as f32 / atlas_height as f32;
        self.uv[2] = (self.y + self.height) as f32 / atlas_height as f32;
        self.uv[3] = self.y as f32 / atlas_height as f32;
    }

    pub(crate) fn get_uv(&self) -> [f32; 4] {
        [0.0; 4]
    }

    pub(crate) fn get_x_offset(&self, height: i32) -> i32 {
        0
    }

    pub(crate) fn get_y_offset(&self, height: i32) -> i32 {
        0
    }

    pub(crate) fn get_x_advance(&self, height: i32) -> i32 {
        0
    
    }

    pub(crate) fn get_width(&self, height: i32) -> i32 {
        0
    }

    pub(crate) fn get_height(&self, height: i32) -> i32 {
        0
    }
}

pub(crate) struct FontLoader {
    core: Rc<RenderCore>
}

impl FontLoader {
    pub(crate) fn new(core: Rc<RenderCore>) -> FontLoader { 
        FontLoader {
            core
        }
    }

    pub(crate) fn load_ttf(&self, contents: Vec<u8>) -> Font {
        todo!()
    }

    pub(crate) fn load_bitmap(&self, texture: Vec<u8>, data: &str) -> Font {
        fn get_attrib(line: &str, name: &str) -> String {
            let re = regex::Regex::new(r"\s+").unwrap();
            let l = re.replace_all(line, " ");
            let attribs = l.split(" ");
            for attrib in attribs.into_iter() {
                if attrib.starts_with(name + "=") {
                    return attrib.split("=").nth(1).unwrap().to_string();
                }
            }
            return String::new();
        }

        fn create_chars(fnt: &str) -> HashMap<char, Glyph> {
            let map = HashMap::new();
            let lines = fnt.split("\n");
            let total_chars: u16 = 0;
            let atlas_width: u16 = 0
            let atlas_height: u16 = 0;
            let line_height: u8 = 0;

            for line in lines {
                if line.contains("count") {
                    total_chars = get_attrib(line, "count").parse::<u16>();
                }
                if line.contains("common") {
                    line_height = get_attrib(line, "lineHeight").parse::<u16>();
                    atlas_width = get_attrib(line, "scaleW").parse::<u16>();
                    atlas_height = get_attrib(line, "scaleH").parse::<u16>();
                }

            }

            todo!()
        }

        todo!()
    }
}