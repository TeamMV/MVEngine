use crate::color::{HsvColor, RgbColor};

#[derive(Debug)]
pub enum ColorParseError {
    InvalidFormat,
    InvalidHex,
    UnexpectedEnd,
    UnexpectedToken
}

pub type Result = core::result::Result<RgbColor, (ColorParseError, String)>;

pub fn parse_color(col: &str) -> Result {
    match col {
        "white" => Ok(RgbColor::white()),
        "black" => Ok(RgbColor::black()),
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

    let mut lexer = ColStrLexer::new(col.to_string());
    let name = match lexer.expect_some(col)? { Token::Name(s) => Ok(s), _ => Err((ColorParseError::UnexpectedToken, "Expected 'Name' Token.".to_string())) }?;

    match name.as_str() {
        "rgb" => parse_rgb_color(lexer, col),
        "rgba" => parse_rgba_color(lexer, col),
        "hsl" => parse_hsl_color(lexer, col),
        "hsla" => parse_hsla_color(lexer, col),
        "hsv" => parse_hsl_color(lexer, col),
        "hsva" => parse_hsla_color(lexer, col),
        _ => Err((ColorParseError::InvalidFormat, format!("Invalid Color format for color '{col}'!").to_string()))
    }
}

fn parse_rgb_color(mut lexer: ColStrLexer, col: &str) -> Result {
    lexer.expect_some(col).and_then(|t| if matches!(t, Token::OpenParen) { Ok(t) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Name' Token.".to_string())) })?;
    let r = lexer.expect_some(col).and_then(|t| if let Token::Lit(l) = t { Ok(l) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Literal' Token.".to_string())) })?;
    lexer.expect_some(col).and_then(|t| if matches!(t, Token::Comma) { Ok(t) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Comma' Token.".to_string())) })?;
    let g = lexer.expect_some(col).and_then(|t| if let Token::Lit(l) = t { Ok(l) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Literal' Token.".to_string())) })?;
    lexer.expect_some(col).and_then(|t| if matches!(t, Token::Comma) { Ok(t) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Comma' Token.".to_string())) })?;
    let b = lexer.expect_some(col).and_then(|t| if let Token::Lit(l) = t { Ok(l) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Literal' Token.".to_string())) })?;
    lexer.expect_some(col).and_then(|t| if matches!(t, Token::CloseParen) { Ok(t) } else { Err((ColorParseError::UnexpectedToken, "Expected 'CloseParen' Token.".to_string())) })?;

    Ok(RgbColor::new([r as u8, g as u8, b as u8, 255]))
}

fn parse_rgba_color(mut lexer: ColStrLexer, col: &str) -> Result {
    lexer.expect_some(col).and_then(|t| if matches!(t, Token::OpenParen) { Ok(t) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Name' Token.".to_string())) })?;
    let r = lexer.expect_some(col).and_then(|t| if let Token::Lit(l) = t { Ok(l) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Literal' Token.".to_string())) })?;
    lexer.expect_some(col).and_then(|t| if matches!(t, Token::Comma) { Ok(t) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Comma' Token.".to_string())) })?;
    let g = lexer.expect_some(col).and_then(|t| if let Token::Lit(l) = t { Ok(l) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Literal' Token.".to_string())) })?;
    lexer.expect_some(col).and_then(|t| if matches!(t, Token::Comma) { Ok(t) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Comma' Token.".to_string())) })?;
    let b = lexer.expect_some(col).and_then(|t| if let Token::Lit(l) = t { Ok(l) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Literal' Token.".to_string())) })?;
    lexer.expect_some(col).and_then(|t| if matches!(t, Token::Comma) { Ok(t) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Comma' Token.".to_string())) })?;
    let a = lexer.expect_some(col).and_then(|t| if let Token::Lit(l) = t { Ok(l) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Literal' Token.".to_string())) })?;
    lexer.expect_some(col).and_then(|t| if matches!(t, Token::CloseParen) { Ok(t) } else { Err((ColorParseError::UnexpectedToken, "Expected 'CloseParen' Token.".to_string())) })?;

    Ok(RgbColor::new([r as u8, g as u8, b as u8, a as u8]))
}

fn parse_hsl_color(mut lexer: ColStrLexer, col: &str) -> Result {
    lexer.expect_some(col).and_then(|t| if matches!(t, Token::OpenParen) { Ok(t) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Name' Token.".to_string())) })?;
    let h = lexer.expect_some(col).and_then(|t| if let Token::Lit(l) = t { Ok(l) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Literal' Token.".to_string())) })?;
    lexer.expect_some(col).and_then(|t| if matches!(t, Token::Comma) { Ok(t) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Comma' Token.".to_string())) })?;
    let s = lexer.expect_some(col).and_then(|t| if let Token::Lit(l) = t { Ok(l) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Literal' Token.".to_string())) })?;
    lexer.expect_some(col).and_then(|t| if matches!(t, Token::Comma) { Ok(t) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Comma' Token.".to_string())) })?;
    let l = lexer.expect_some(col).and_then(|t| if let Token::Lit(l) = t { Ok(l) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Literal' Token.".to_string())) })?;
    lexer.expect_some(col).and_then(|t| if matches!(t, Token::CloseParen) { Ok(t) } else { Err((ColorParseError::UnexpectedToken, "Expected 'CloseParen' Token.".to_string())) })?;

    Ok(HsvColor::new([h, l, s, 0.0]).to_rgb())
}

fn parse_hsla_color(mut lexer: ColStrLexer, col: &str) -> Result {
    lexer.expect_some(col).and_then(|t| if matches!(t, Token::OpenParen) { Ok(t) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Name' Token.".to_string())) })?;
    let h = lexer.expect_some(col).and_then(|t| if let Token::Lit(l) = t { Ok(l) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Literal' Token.".to_string())) })?;
    lexer.expect_some(col).and_then(|t| if matches!(t, Token::Comma) { Ok(t) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Comma' Token.".to_string())) })?;
    let s = lexer.expect_some(col).and_then(|t| if let Token::Lit(l) = t { Ok(l) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Literal' Token.".to_string())) })?;
    lexer.expect_some(col).and_then(|t| if matches!(t, Token::Comma) { Ok(t) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Comma' Token.".to_string())) })?;
    let l = lexer.expect_some(col).and_then(|t| if let Token::Lit(l) = t { Ok(l) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Literal' Token.".to_string())) })?;
    lexer.expect_some(col).and_then(|t| if matches!(t, Token::Comma) { Ok(t) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Comma' Token.".to_string())) })?;
    let a = lexer.expect_some(col).and_then(|t| if let Token::Lit(l) = t { Ok(l) } else { Err((ColorParseError::UnexpectedToken, "Expected 'Literal' Token.".to_string())) })?;
    lexer.expect_some(col).and_then(|t| if matches!(t, Token::CloseParen) { Ok(t) } else { Err((ColorParseError::UnexpectedToken, "Expected 'CloseParen' Token.".to_string())) })?;

    Ok(HsvColor::new([h, s, l, a]).to_rgb())
}

fn parse_hex_color(col: &str) -> Result {
    let mut new_col = match col.len() {
        3 | 4 => {
            col
                .chars()
                .map(|c| [c, c])
                .flatten()
                .collect::<String>()
        }
        _ => col.to_string()
    };

    if new_col.len() == 6 {
        let mut s = new_col;
        s.push_str("FF");
        let s = s.to_uppercase();
        new_col = s;
    }

    let bits = u32::from_str_radix(new_col.as_str(), 16)
        .map_err(|_| (ColorParseError::InvalidHex, format!("Invalid Hex string '{new_col}'!").to_string()))?;
    Ok(RgbColor::new(bits.to_be_bytes()))
}

enum Token {
    Name(String),
    OpenParen,
    CloseParen,
    Lit(f32),
    Comma
}
struct ColStrLexer {
    src: String,
    idx: usize
}

impl ColStrLexer {
    pub fn new(src: String) -> Self {
        Self {
            src,
            idx: 0,
        }
    }

    pub fn expect_some(&mut self, col: &str) -> core::result::Result<Token, (ColorParseError, String)> {
        let next = self.next();
        next.ok_or((ColorParseError::UnexpectedEnd, format!("Unexpected end of color string '{col}'").to_string()))
    }

    pub fn next(&mut self) -> Option<Token> {
        let next = self.src.chars().nth(self.idx);
        if next.is_none() { return None; }
        let mut next = next.unwrap();
        while next.is_whitespace() {
            self.idx += 1;

            let next_opt = self.src.chars().nth(self.idx);
            if next_opt.is_none() { return None; }
            next = next_opt.unwrap();
        }
        self.idx += 1;
        match next {
            '(' => Some(Token::OpenParen),
            ')' => Some(Token::CloseParen),
            ',' => Some(Token::Comma),

            _ => {
                if next.is_numeric() {
                    let mut str = String::new();
                    str.push(next);
                    while let Some(next) = self.src.chars().nth(self.idx) {
                        if next.is_numeric() || next == '.' {
                            self.idx += 1;
                            str.push(next);
                        } else {
                            break;
                        }
                    }
                    let lit: f32 = str.parse().ok()?;
                    Some(Token::Lit(lit))
                } else {
                    let mut str = String::new();
                    str.push(next);
                    while let Some(next) = self.src.chars().nth(self.idx) {
                        if next.is_alphanumeric() || next == '_' {
                            self.idx += 1;
                            str.push(next);
                        } else {
                            break;
                        }
                    }
                    Some(Token::Name(str))
                }
            }
        }
    }
}