use hashbrown::HashMap;
use std::cmp::max;
use std::hash::{BuildHasher, Hasher};
use std::sync::Arc;
use std::time::Instant;

use mvutils::utils::TetrahedronOp;

use crate::render::common::Texture;
use crate::render::init::State;

pub struct TypeFace {
    pub regular: Arc<Font>,
    pub bold: Arc<Font>,
    pub italic: Arc<Font>,
    pub italic_bold: Arc<Font>,
}

impl TypeFace {
    pub fn single(font: Arc<Font>) -> Arc<Self> {
        Arc::new(TypeFace {
            regular: font.clone(),
            bold: font.clone(),
            italic: font.clone(),
            italic_bold: font,
        })
    }
}

pub struct Font {
    textures: Vec<Arc<Texture>>,
    alphabet: HashMap<char, Glyph, CharIdentityHasher>,

    max_height: u16,
    max_width: u16,

    kernings: HashMap<u64, i8, IdentityHasher>,
}

impl Font {
    pub(crate) fn get_texture(&self, idx: usize) -> Arc<Texture> {
        self.textures.get(idx).expect("idx out of bounds").clone()
    }

    pub(crate) fn supports(&self, c: char) -> bool {
        self.alphabet.contains_key(&c)
    }

    pub(crate) fn get_glyph(&self, c: char) -> &Glyph {
        self.alphabet
            .get(&c)
            .unwrap_or_else(|| panic!("The char '{}' is not supported in this font!", c))
    }

    pub(crate) fn get_kerning(&self, first: char, second: char) -> i8 {
        let pos: u64 = first as u64 | (second as u64) << 32;
        self.kernings.get(&pos).copied().unwrap_or(0)
    }

    pub fn get_metrics(&self, string: &str) -> FontMetrics {
        FontMetrics {
            font: self,
            string: string.to_string(),
        }
    }
}

pub struct FontMetrics {
    font: *const Font,
    string: String,
}

impl FontMetrics {
    pub fn fits(&self, width: i32, height: i32) -> i32 {
        todo!()
    }

    pub fn width(&self, height: i32) -> i32 {
        let mut ret = 0;
        for c in self.string.chars() {
            ret += self.char_width(height, c);
        }
        ret
    }

    pub fn char_width(&self, height: i32, c: char) -> i32 {
        if height == 0 {
            return 0;
        }
        let glyph = unsafe { self.font.as_ref().unwrap().get_glyph(c) };
        glyph.get_width(height)
            * (height / (glyph.get_height(height) == 0).yn(1, glyph.get_height(height)))
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
    x_adv: i16,
    page: u16,

    max_height: u16,
}

#[allow(clippy::too_many_arguments)]
impl Glyph {
    fn new(
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        x_off: i16,
        y_off: i16,
        x_adv: i16,
        page: u16,
        chnl: u8,
        c: char,
    ) -> Self {
        Glyph {
            uv: [0.0; 4],
            c,
            x,
            y,
            width,
            height,
            x_off,
            y_off,
            x_adv,
            page,
            max_height: 0,
        }
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
        self.multiply(height, self.x_off as i32)
    }

    pub(crate) fn get_y_offset(&self, height: i32) -> i32 {
        self.multiply(height, self.y_off as i32)
    }

    pub(crate) fn get_x_advance(&self, height: i32) -> i32 {
        self.multiply(height, self.x_adv as i32)
    }

    pub(crate) fn get_width(&self, height: i32) -> i32 {
        self.multiply(height, self.width as i32)
    }

    pub(crate) fn get_height(&self, height: i32) -> i32 {
        self.multiply(height, self.height as i32)
    }

    pub(crate) fn get_page(&self) -> u16 {
        self.page
    }

    pub(crate) fn multiply(&self, height: i32, value: i32) -> i32 {
        (height as f32 / self.max_height as f32 * value as f32).floor() as i32
    }
}

#[derive(Clone)]
pub(crate) struct FontLoader;

impl FontLoader {
    pub(crate) fn new() -> FontLoader {
        FontLoader
    }

    pub(crate) fn load_default_font(&self, state: &State) -> Font {
        self.load_bitmap(
            state,
            &[
                include_bytes!("fonts/roboto1.png"),
                include_bytes!("fonts/roboto2.png"),
                include_bytes!("fonts/roboto3.png"),
                include_bytes!("fonts/roboto4.png"),
                include_bytes!("fonts/roboto5.png"),
            ],
            include_str!("fonts/roboto.fnt"),
        )
    }

    pub(crate) fn load_ttf(&self, contents: Vec<u8>) -> Font {
        todo!()
    }

    pub(crate) fn load_bitmap(&self, state: &State, textures: &[&[u8]], data: &str) -> Font {
        let mut kernings = HashMap::with_hasher(IdentityHasher::default());
        let mut map = HashMap::with_hasher(CharIdentityHasher::default());
        let lines = data.split('\n');
        let mut atlas_width: u16 = 0;
        let mut atlas_height: u16 = 0;
        let mut max_width = 0;
        let mut max_height = 0;
        let mut max_x_off = 0;
        let mut max_y_off = 0;
        let mut packed = false;

        for line in lines {
            if line.starts_with("common ") {
                for attrib in line.split_whitespace() {
                    if !attrib.contains('=') {
                        continue;
                    }
                    let (name, value) = attrib.split_once('=').unwrap();
                    match name {
                        "scaleW" => atlas_width = value.parse::<u16>().unwrap_or(0),
                        "scaleH" => atlas_height = value.parse::<u16>().unwrap_or(0),
                        "packed" => packed = value == "1",
                        _ => {}
                    }
                }
            } else if line.starts_with("char ") {
                let mut c = 0 as char;
                let mut x = 0;
                let mut y = 0;
                let mut width = 0;
                let mut height = 0;
                let mut xoffset = 0;
                let mut yoffset = 0;
                let mut xadvance = 0;
                let mut page = 0;
                let mut chnl = 0;
                for attrib in line.split_whitespace() {
                    if !attrib.contains('=') {
                        continue;
                    }
                    let (name, value) = attrib.split_once('=').unwrap();
                    match name {
                        "id" => {
                            c = if let Ok(Some(c)) = value.parse::<u32>().map(char::from_u32) {
                                c
                            } else {
                                continue;
                            }
                        }
                        "x" => x = value.parse::<u16>().unwrap_or(0),
                        "y" => y = value.parse::<u16>().unwrap_or(0),
                        "width" => {
                            width = value.parse::<u16>().unwrap_or(0);
                            max_width = max_width.max(width);
                        }
                        "height" => {
                            height = value.parse::<u16>().unwrap_or(0);
                            max_height = max_height.max(height);
                        }
                        "xoffset" => {
                            xoffset = value.parse::<i16>().unwrap_or(0);
                            max_x_off = max_x_off.max(xoffset);
                        }
                        "yoffset" => {
                            yoffset = value.parse::<i16>().unwrap_or(0);
                            max_y_off = max_y_off.max(yoffset);
                        }
                        "xadvance" => xadvance = value.parse::<i16>().unwrap_or(0),
                        "page" => page = value.parse::<u16>().unwrap_or(0),
                        "chnl" => chnl = value.parse::<u8>().unwrap_or(0),
                        _ => {}
                    }
                }
                let glyph = Glyph::new(
                    x, y, width, height, xoffset, yoffset, xadvance, page, chnl, c,
                );
                map.insert(c, glyph);
            } else if line.starts_with("kerning ") {
                let mut first = 0;
                let mut second = 0;
                let mut amount = 0;

                for attrib in line.split_whitespace() {
                    if !attrib.contains('=') {
                        continue;
                    }
                    let (name, value) = attrib.split_once('=').unwrap();
                    match name {
                        "first" => {
                            first = if let Ok(v) = value.parse::<u32>() {
                                v
                            } else {
                                continue;
                            }
                        }
                        "second" => {
                            second = if let Ok(v) = value.parse::<u32>() {
                                v
                            } else {
                                continue;
                            }
                        }
                        "amount" => {
                            amount = if let Ok(v) = value.parse::<i8>() {
                                if v == 0 {
                                    continue;
                                }
                                v
                            } else {
                                continue;
                            }
                        }
                        _ => {}
                    }
                }

                let pos: u64 = first as u64 | (second as u64) << 32;

                kernings.insert(pos, amount);
            }
        }

        for glyph in map.values_mut() {
            glyph.make_coords(atlas_width, atlas_height, max_height);
        }

        let mut tex_vec = Vec::with_capacity(textures.len());
        for tex in textures {
            let mut texture = Texture::new(tex.to_vec());
            texture.make(state);
            tex_vec.push(Arc::new(texture));
        }

        Font {
            textures: tex_vec,
            alphabet: map,
            max_width,
            max_height,
            kernings,
        }
    }
}

#[derive(Default)]
struct IdentityHasher {
    value: u64,
}

impl Hasher for IdentityHasher {
    fn finish(&self) -> u64 {
        self.value
    }

    fn write(&mut self, bytes: &[u8]) {
        unreachable!()
    }

    fn write_u64(&mut self, i: u64) {
        self.value = i
    }
}

impl BuildHasher for IdentityHasher {
    type Hasher = Self;

    fn build_hasher(&self) -> Self::Hasher {
        Self::default()
    }
}

#[derive(Default)]
struct CharIdentityHasher {
    value: u32,
}

impl Hasher for CharIdentityHasher {
    fn finish(&self) -> u64 {
        self.value as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        unreachable!()
    }

    fn write_u32(&mut self, i: u32) {
        self.value = i;
    }
}

impl BuildHasher for CharIdentityHasher {
    type Hasher = Self;

    fn build_hasher(&self) -> Self::Hasher {
        Self::default()
    }
}
