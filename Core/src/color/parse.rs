use crate::color::RgbColor;

pub enum ColorParseError {
    InvalidFormat,
    InvalidHex
}

pub type Result = core::result::Result<RgbColor, (ColorParseError, String)>;

pub fn parse_color(col: &str) -> Result {
    match col {
        "white" => Ok(RgbColor::white()),
        "back" => Ok(RgbColor::black()),
        "red" => Ok(RgbColor::red()),
        "green" => Ok(RgbColor::green()),
        "blue" => Ok(RgbColor::blue()),
        "yellow" => Ok(RgbColor::yellow()),
        "magenta" => Ok(RgbColor::magenta()),
        "cyan" => Ok(RgbColor::cyan()),
        "transparent" => Ok(RgbColor::transparent()),
        _ => route_parser(col)
    }
}

fn route_parser(col: &str) -> Result {
    if col.starts_with("#") {
        return parse_hex_color(&col[1..]);
    }
    if col.starts_with("0x") {
        return parse_hex_color(&col[2..]);
    }

    Err((ColorParseError::InvalidFormat, format!("Invalid Color format for color '{col}'!").to_string()))
}

fn parse_hex_color(col: &str) -> Result {
    // match col.len() {
    //     3 | 6 => {
    //
    //     }
    //
    //
    //     _ => {}
    // }
    todo!()
}

fn parse_rgb_color(col: &str) -> Result {
    todo!()
}

enum Token {
    Name(String),
    OpenParen,
    CloseParen,
    Lit(f32),
    Comma
}
struct ColStrLexer {

}