use std::collections::VecDeque;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
#[repr(u8)]
pub enum Token {
    Identifier(String),
    Equals,
    LBracket,
    RBracket,
    Number(NumberLit),
    Selector(String),
    Error(String),
    Semicolon,
}

impl Token {
    pub fn ord(&self) -> u8 {
        let ptr_to_option = (self as *const Token) as *const u8;
        unsafe {
            *ptr_to_option
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum NumberLit {
    Int(i32),
    Float(f32)
}

impl NumberLit {
    pub(crate) fn as_f32(&self) -> f32 {
        match self {
            NumberLit::Int(i) => *i as f32,
            NumberLit::Float(f) => *f
        }
    }

    pub(crate) fn as_i32(&self) -> i32 {
        match self {
            NumberLit::Int(i) => *i,
            NumberLit::Float(f) => *f as i32
        }
    }
}

pub struct TokenStream<'a> {
    chars: Peekable<Chars<'a>>,
    in_struct: bool,
    bracket_depth: i32,
    putback: VecDeque<Token>
}

impl<'a> TokenStream<'a> {
    pub fn tokenize(shape_expr: &'a str) -> Self {
        Self {
            chars: shape_expr.chars().peekable(),
            in_struct: false,
            bracket_depth: 0,
            putback: VecDeque::new(),
        }
    }

    pub fn putback(&mut self, token: Token) {
        self.putback.push_back(token);
    }

    fn parse_next_str(&mut self, start: Option<char>, allow_numbers: bool) -> String {
        let mut s = String::new();
        if let Some(start) = start {
            s.push(start);
        }
        if self.chars.peek().is_some_and(|c| !Self::char_str_valid(*c, allow_numbers)) { return s; }
        while let Some(next) = self.chars.next() {
            s.push(next);
            if self.chars.peek().is_some_and(|c| !Self::char_str_valid(*c, allow_numbers)) { return s; }
        }
        s
    }

    fn char_str_valid(c: char, allow_nums: bool) -> bool {
        if allow_nums {
            c.is_alphanumeric() || c == '_'
        } else {
            c.is_alphabetic() || c == '_'
        }
    }

    fn parse_number(&mut self, start: char) -> Option<NumberLit> {
        let mut num_str = String::new();
        num_str.push(start);
        if self.chars.peek().is_some_and(|c| !c.is_numeric() && *c != '.' && *c != '-') { return self.get_lit_from_str(num_str); }
        while let Some(next) = self.chars.next() {
            num_str.push(next);
            if self.chars.peek().is_some_and(|c| !c.is_numeric() && *c != '.' && *c != '-') { return self.get_lit_from_str(num_str); }
        }
        self.get_lit_from_str(num_str)
    }

    fn get_lit_from_str(&self, s: String) -> Option<NumberLit> {
        if s.contains('.') {
            s.parse::<f32>().ok().map(|f| NumberLit::Float(f))
        } else {
            s.parse::<i32>().ok().map(|i| NumberLit::Int(i))
        }
    }

    pub fn expect_next_ident(&mut self) -> Result<String, String> {
        if let Some(next) = self.next() {
            if let Token::Identifier(ident) = next {
                Ok(ident)
            } else {
                Err(format!("Expected Identifier, got {:?}", next))
            }
        } else {
            Err("Expected Identifier, got EOF".to_string())
        }
    }

    pub fn expect_next_token(&mut self, token: Token) -> Result<(), String> {
        if let Some(next) = self.next() {
            if token.ord() == next.ord() {
                Ok(())
            } else {
                Err(format!("Expected {:?}, got {:?}", token, next))
            }
        } else {
            Err(format!("Expected {:?}, got EOF", token))
        }
    }

    pub fn expect_next_some(&mut self) -> Result<Token, String> {
        if let Some(next) = self.next() {
            Ok(next)
        } else {
            Err("Expected Some(...), got EOF".to_string())
        }
    }
}

impl Iterator for TokenStream<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.putback.is_empty() {
            return self.putback.pop_front();
        }

        let next = self.chars.next();
        if next.is_none() {
            return None;
        }
        let mut next = next.unwrap();
        while next.is_whitespace() {
            let n = self.chars.next();
            if n.is_none() { return None }
            next = n.unwrap();
        }
        if next == '#' {
            while next != '\n' {
                let n = self.chars.next();
                if n.is_none() { return None }
                next = n.unwrap();
            }
            while next.is_whitespace() {
                let n = self.chars.next();
                if n.is_none() { return None }
                next = n.unwrap();
            }
        }

        match next {
            '=' => Some(Token::Equals),
            ';' => Some(Token::Semicolon),
            '[' => {
                self.in_struct = true;
                self.bracket_depth += 1;
                Some(Token::LBracket)
            },
            ']' => {
                self.bracket_depth -= 1;
                if self.bracket_depth <= 0 {
                    self.in_struct = false;
                }
                Some(Token::RBracket)
            },
            '>' => {
                let s = self.parse_next_str(None, true);
                Some(Token::Selector(s))
            }
            _ => {
                if next.is_numeric() || next == '-' {
                    let lit = self.parse_number(next);
                    return if lit.is_some() {
                        Some(Token::Number(lit.unwrap()))
                    } else {
                        Some(Token::Error("Unable to parse number".to_string()))
                    }
                }
                let s = self.parse_next_str(Some(next), !self.in_struct);
                Some(Token::Identifier(s))
            }
        }
    }
}