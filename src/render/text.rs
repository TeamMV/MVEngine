use std::{cell::RefCell, rc::Rc, collections::HashMap};
use std::cmp::max;

use super::{shared::Texture, RenderCore};

pub struct TypeFace {
    pub regular: Rc<Font>,
    pub bold: Rc<Font>,
    pub italic: Rc<Font>,
    pub italic_bold: Rc<Font>,
}

impl TypeFace {
    pub fn single(font: Rc<Font>) -> Rc<Self> {
        Rc::new(TypeFace {
            regular: font.clone(),
            bold: font.clone(),
            italic: font.clone(),
            italic_bold: font.clone(),
        })
    }
}

pub struct Font {
    texture: Rc<RefCell<Texture>>,
    alphabet: HashMap<char, Glyph>,

    max_height: u16,
    max_width: u16
}

impl Font {
    pub(crate) fn get_texture(&self) -> Rc<RefCell<Texture>> {
        self.texture.clone()
    }

    pub(crate) fn supports(&self, c: char) -> bool {
        self.alphabet.contains_key(&c)
    }

    pub(crate) fn get_glyph(&self, c: char) -> &Glyph {
        self.alphabet.get(&c).expect(format!("The char '{}' is not supported in this font!", c).as_str())
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
        if height == 0 {return 0;}
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
    x_off: i16,
    y_off: i16,
    x_adv: u16,

    max_height: u16
}

#[allow(clippy::too_many_arguments)]
impl Glyph {
    fn new(x: u16, y: u16, width: u16, height: u16, x_off: i16, y_off: i16, x_adv: u16, c: char) -> Self {
        Glyph { uv: [0.0; 4], c, x, y, width, height, x_off, y_off, x_adv, max_height: 0}
    }

    fn make_coords(&mut self, atlas_width: u16, atlas_height: u16, max_height: u16) {
        self.uv[0] = self.x as f32 / atlas_width as f32; //x0
        self.uv[1] = (self.x + self.width) as f32 / atlas_height as f32; //x1
        self.uv[2] = (self.y + self.height) as f32 / atlas_height as f32; //y0
        self.uv[3] = self.y as f32 / atlas_height as f32; //y1

        self.max_height = max_height;
    }

    pub(crate) fn get_uv(&self) -> [f32; 4] {
        self.uv
    }

    pub(crate) fn get_x_offset(&self, height: i32) -> i32 {
        self.multiplier(height, self.x_off as i32)
    }

    pub(crate) fn get_y_offset(&self, height: i32) -> i32 {
        self.multiplier(height, self.y_off as i32)
    }

    pub(crate) fn get_x_advance(&self, height: i32) -> i32 {
        self.multiplier(height, self.x_adv as i32)
    }

    pub(crate) fn get_width(&self, height: i32) -> i32 {
        self.multiplier(height, self.width as i32)
    }

    pub(crate) fn get_height(&self, height: i32) -> i32 {
        self.multiplier(height, self.height as i32)
    }

    fn multiplier(&self, height: i32, value: i32) -> i32 {
        (height as f32 / self.max_height as f32 * value as f32) as i32
    }
}

#[derive(Clone)]
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

    pub(crate) fn load_bitmap(&self, texture: &[u8], data: &str) -> Font {
        fn get_attrib(line: &str, name: &str) -> String {
            let re = regex::Regex::new(r"\s+").unwrap();
            let l = re.replace_all(line, " ");
            for attrib in l.split_whitespace().into_iter() {
                if attrib.starts_with(name) {
                    return attrib.split("=").nth(1).unwrap_or("0").to_string();
                }
            }
            "0".to_string()
        }

        let mut map = HashMap::new();
        let lines = data.split("\n");
        let mut total_chars: u16 = 0;
        let mut atlas_width: u16 = 0;
        let mut atlas_height: u16 = 0;
        let mut line_height: u8 = 0;
        let mut max_width = 0;
        let mut max_height = 0;
        let mut max_x_off = 0;
        let mut max_y_off = 0;

        for line in lines {
            if line.contains("chars ") {
                total_chars = get_attrib(line, "count").parse::<u16>().unwrap();
            }
            if line.contains("common ") {
                line_height = get_attrib(line, "lineHeight").parse::<u8>().unwrap();
                atlas_width = get_attrib(line, "scaleW").parse::<u16>().unwrap();
                atlas_height = get_attrib(line, "scaleH").parse::<u16>().unwrap();
            }
            if total_chars != 0 {
                //individual character parsing
                max_width = max(max_width, get_attrib(line, "width").parse::<u16>().unwrap());
                max_height = max(max_height, get_attrib(line, "height").parse::<u16>().unwrap());
                max_x_off = max(max_x_off, get_attrib(line, "xoffset").parse::<i16>().unwrap());
                max_y_off = max(max_y_off, get_attrib(line, "yoffset").parse::<i16>().unwrap());
                let c = char::from_u32(get_attrib(line, "id").parse::<u32>().unwrap()).unwrap();
                let glyph = Glyph::new(
                    get_attrib(line, "x").parse::<u16>().unwrap(),
                    get_attrib(line, "y").parse::<u16>().unwrap(),
                    get_attrib(line, "width").parse::<u16>().unwrap(),
                    get_attrib(line, "height").parse::<u16>().unwrap(),
                    get_attrib(line, "xoffset").parse::<i16>().unwrap(),
                    get_attrib(line, "yoffset").parse::<i16>().unwrap(),
                    get_attrib(line, "xadvance").parse::<u16>().unwrap(),
                    c
                );
                map.insert(c, glyph);
            }
        }

        for glyph in map.values_mut() {
            glyph.make_coords(atlas_width, atlas_height, max_height);
        }

        let texture = self.core.create_texture(texture);

        Font {
            texture: Rc::new(RefCell::new(texture)),
            alphabet: map,
            max_width,
            max_height
        }
    }
}