use alloc::rc::Rc;
use mvutils::utils::RcMut;
use crate::render::draw::Draw2D;
use crate::render::text::{Font, TypeFace};

use bitflags::{bitflags};
use mvutils::deref;
use mvutils::serialize::{Deserializer, Serializable, Serializer};
use crate::render::color::{Color, Gradient, RGB};

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct FontStyle: u8 {
        const REGULAR = 0;
        const BOLD = 1;
        const ITALIC = 2;
        const UNDERLINE = 4;
        const STRIKETHROUGH = 8;
        const OBFUSCATED = 16;
        const CHROMA = 32;
    }
}

impl FontStyle {
    pub fn is_underline(&self) -> bool {
        self.bits() & FontStyle::UNDERLINE.bits() != 0
    }

    pub fn is_strikethrough(&self) -> bool {
        self.bits() & FontStyle::STRIKETHROUGH.bits() != 0
    }

    pub fn is_obfuscated(&self) -> bool {
        self.bits() & FontStyle::OBFUSCATED.bits() != 0
    }

    pub fn is_chroma(&self) -> bool {
        self.bits() & FontStyle::CHROMA.bits() != 0
    }

    pub(crate) fn raw(&self) -> u8 {
        self.bits()
    }

    pub(crate) fn set_raw(&mut self, val: u8) {
        *self.0.bits_mut() = val;
    }
}

impl Default for FontStyle {
    #[inline]
    fn default() -> FontStyle {
        FontStyle::REGULAR
    }
}

fn get_font(face: Rc<TypeFace>, style: FontStyle) -> Rc<Font> {
    match style.bits() & 3 {
        0 => face.regular.clone(),
        1 => face.bold.clone(),
        2 => face.italic.clone(),
        3 => face.italic_bold.clone(),
        _ => unreachable!()
    }
}

#[derive(Default)]
pub struct FormattedString {
    pub pieces: Vec<Format>,
    pub whole: String,
}

impl FormattedString {
    pub fn new(src: &str) -> Self {
        FormattedString {
            pieces: vec![Format {
                style: FontStyle::REGULAR,
                text: src.to_string(),
                color: None,
            }],
            whole: src.to_string(),
        }
    }

    pub fn draw(&self, ctx: RcMut<Draw2D>, x: i32, y: i32, height: i32, font: Option<Rc<TypeFace>>, rotation: f32, rx: i32, ry: i32, col: Gradient<RGB, f32>) {
        let mut char_x = x;
        for fmt in self.pieces.iter() {
            if fmt.color.is_some() {
                ctx.borrow_mut().color(deref!(fmt.color.clone().unwrap()));
            } else {
                ctx.borrow_mut().get_mut_gradient().copy_of(col);
            }
            let font = font.clone().unwrap_or(TypeFace::single(Draw2D::get_default_font(ctx.as_ptr())));
            ctx.borrow_mut().custom_text_origin_rotated(fmt.style.is_chroma(), char_x, y, height, fmt.text.as_str(), get_font(font.clone(), fmt.style), rotation, rx, ry);
            char_x += get_font(font.clone(), fmt.style).get_metrics(fmt.text.as_str()).width(height);
        }
    }
}

impl Serializable for FormattedString {
    fn serialize(&self, serializer: &mut impl Serializer) {
        serializer.push_u64(self.pieces.len() as u64);
        for piece in self.pieces.iter() {
            piece.serialize(serializer);
        }
    }

    fn deserialize(deserializer: &mut impl Deserializer) -> Result<Self, String> {
        let mut pieces = Vec::new();
        let mut whole = String::new();
        let amount = deserializer.pop_u64().ok_or("Invalid formatted string format!".to_string())?;
        for _ in 0..amount {
            let part = Format::deserialize(deserializer)?;
            whole.push_str(part.text.as_str());
            pieces.push(part);
        }
        Ok(Self {
            pieces,
            whole,
        })
    }
}

pub struct Format {
    pub style: FontStyle,
    pub text: String,
    pub color: Option<Rc<Color<RGB, f32>>>,
}

impl Serializable for Format {
    fn serialize(&self, serializer: &mut impl Serializer) {
        serializer.push_u8(self.style.raw());
        serializer.push_string(self.text.as_str());
        if self.color.is_some() {
            let col = self.color.clone().unwrap();
            serializer.push_f32(col.r());
            serializer.push_f32(col.g());
            serializer.push_f32(col.b());
            serializer.push_f32(col.a());
        }
    }

    fn deserialize(deserializer: &mut impl Deserializer) -> Result<Self, String> {
        let mut style = FontStyle::default();
        style.set_raw(deserializer.pop_u8().ok_or("Invalid formatted string piece format!".to_string())?);
        let text = deserializer.pop_string().ok_or("Invalid formatted string piece format!".to_string())?;
        let r = deserializer.pop_f32().ok_or("Invalid formatted string piece format!".to_string())?;
        let g = deserializer.pop_f32().ok_or("Invalid formatted string piece format!".to_string())?;
        let b = deserializer.pop_f32().ok_or("Invalid formatted string piece format!".to_string())?;
        let a = deserializer.pop_f32().ok_or("Invalid formatted string piece format!".to_string())?;
        let color = Rc::new(Color::<RGB, f32>::new(r, g, b, a));
        Ok(Format { style, text, color: Some(color) })
    }
}