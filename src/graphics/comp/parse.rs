// remove when this file is also relevant and like not neglected
#![allow(unused)]

use crate::math::vec::Vec2;
use hashbrown::HashMap;
use std::collections::VecDeque;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
pub struct ParsedAnimation {
    pub(crate) name: String,
    pub(crate) pre: Vec<ParsedKeyframe>,
    pub(crate) keyframes: Vec<(f32, ParsedKeyframe)>,
}

#[derive(Debug)]
pub struct ParsedPart {
    pub(crate) parent: Option<String>,
    pub(crate) name: String,
    pub(crate) index: usize,
    pub(crate) bounds: Vec2,
}

#[derive(Debug)]
pub struct MRFParser {
    pub(crate) parts: Vec<ParsedPart>,
    pub(crate) anims: Vec<ParsedAnimation>,
}

impl MRFParser {
    pub fn parse(expression: &str) -> Result<Self, String> {
        let mut tokens = TokenStream::tokenize(expression);

        let mut state = 0;
        let mut parts = HashMap::new();
        let mut bounds = HashMap::new();
        let mut keyframes = HashMap::new();
        let mut current_keyframes = Vec::new();
        let mut pres = HashMap::new();
        let mut constraints = HashMap::new();
        let mut current_pre = Vec::new();
        let mut current_anim = String::new();
        let mut current_perc = 0f32;
        let mut anim_state = 0;
        while let Some(token) = tokens.next() {
            match token {
                Token::Parts => {
                    if state == 0 {
                        state = 1;
                    } else {
                        return Err("Can only have 1 PARTS section!".to_string());
                    }
                }
                Token::Bounds => {
                    if state == 0 {
                        return Err("Expected BOUNDS after PARTS".to_string());
                    }
                    state = 2;
                }
                Token::Animations => {
                    if state == 0 {
                        return Err("Expected ANIMATIONS after PARTS".to_string());
                    }
                    state = 3;
                }
                Token::Constraints => {
                    if state == 0 {
                        return Err("Expected CONSTRAINTS after PARTS".to_string());
                    }
                    state = 4;
                }
                Token::Ident(s) => {
                    if state == 1 {
                        tokens.expect_next_token(Token::Equals)?;
                        let idx = tokens.expect_next_some()?;
                        if let Token::Number(n) = idx {
                            let idx = n as usize;
                            parts.insert(s, idx);
                        } else {
                            return Err("Expected Number for Ident in PARTS!".to_string());
                        }
                    } else if state == 2 {
                        tokens.expect_next_token(Token::Equals)?;
                        let b = tokens.expect_next_some()?;
                        if let Token::Vec2(vec2) = b {
                            bounds.insert(s, vec2);
                        } else {
                            return Err("Expected Vec2 for Ident in BOUNDS!".to_string());
                        }
                    } else if state == 3 {
                        if anim_state == 0 {
                            tokens.putback(Token::Ident(s));
                            let (name, path) = Self::parse_path(&mut tokens)?;
                            tokens.expect_next_token(Token::Equals)?;
                            let keyframe = match path {
                                Path::Translate | Path::Scale | Path::Origin => {
                                    let x = Self::parse_vec2(&mut tokens)?;
                                    ParsedKeyframe::from_vec(path, x, name)
                                }
                                Path::TranslateX
                                | Path::TranslateY
                                | Path::ScaleX
                                | Path::ScaleY
                                | Path::OriginX
                                | Path::OriginY
                                | Path::Rotate => {
                                    let x = Self::parse_f32(&mut tokens)?;
                                    ParsedKeyframe::from_num(path, x, name)
                                }
                            };
                            current_keyframes.push((current_perc, keyframe))
                        } else if anim_state == 1 {
                            tokens.putback(Token::Ident(s));
                            let (name, path) = Self::parse_path(&mut tokens)?;
                            tokens.expect_next_token(Token::Equals)?;
                            let keyframe = match path {
                                Path::Translate | Path::Scale | Path::Origin => {
                                    let x = Self::parse_vec2(&mut tokens)?;
                                    ParsedKeyframe::from_vec(path, x, name)
                                }
                                Path::TranslateX
                                | Path::TranslateY
                                | Path::ScaleX
                                | Path::ScaleY
                                | Path::OriginX
                                | Path::OriginY
                                | Path::Rotate => {
                                    let x = Self::parse_f32(&mut tokens)?;
                                    ParsedKeyframe::from_num(path, x, name)
                                }
                            };
                            current_pre.push(keyframe);
                        }
                    } else if state == 4 {
                        tokens.expect_next_token(Token::Arrow)?;
                        let child = tokens.expect_next_ident()?;
                        constraints.insert(s, child);
                    }
                }
                Token::New(anim) => {
                    if state != 3 {
                        return Err(
                            "Animations can only be created in the ANIMATIONS section!".to_string()
                        );
                    }
                    anim_state = 0;
                    current_anim = anim;
                }
                Token::Finish => {
                    keyframes.insert(current_anim.clone(), current_keyframes);
                    pres.insert(current_anim.clone(), current_pre);
                    current_pre = Vec::new();
                    current_keyframes = Vec::new();
                }
                Token::Keyframe(k) => {
                    current_perc = k;
                }
                Token::Pre => {
                    anim_state = 1;
                }
                Token::Keyframes => {
                    anim_state = 0;
                }
                _ => {
                    return Err(format!("Unexpected token '{:?}'!", token));
                }
            }
        }

        let mut parts_vec = Vec::new();
        for (name, index) in parts {
            let bounds = *bounds.get(&name).expect("should be here");
            let parent = constraints.get(&name).cloned();
            let part = ParsedPart {
                parent,
                name,
                index,
                bounds,
            };
            parts_vec.push(part);
        }
        let mut anims = Vec::new();
        for (name, vec) in keyframes {
            let pre = pres.remove(&name).expect("should be here");
            let anim = ParsedAnimation {
                name,
                pre,
                keyframes: vec,
            };
            anims.push(anim);
        }

        Ok(Self {
            parts: parts_vec,
            anims,
        })
    }

    fn parse_f32(tokens: &mut TokenStream) -> Result<f32, String> {
        let next = tokens.expect_next_some()?;
        if let Token::Number(f) = next {
            Ok(f)
        } else {
            tokens.putback(next);
            Err("Expected a number!".to_string())
        }
    }

    fn parse_vec2(tokens: &mut TokenStream) -> Result<Vec2, String> {
        let next = tokens.expect_next_some()?;
        if let Token::Vec2(v) = next {
            Ok(v)
        } else {
            tokens.putback(next);
            Err("Expected a vec2!".to_string())
        }
    }

    fn parse_path(tokens: &mut TokenStream) -> Result<(String, Path), String> {
        if let Some(Token::Ident(name)) = tokens.next() {
            let next = tokens
                .next()
                .ok_or("Unexpected end of stream!".to_string())?;
            if let Token::Dot = next {
                let trans = tokens.expect_next_ident()?;
                return match trans.as_str() {
                    "translation" => {
                        let subpath = Self::parse_sub_path(tokens)?;
                        match subpath {
                            SubPath::None => Ok((name, Path::Translate)),
                            SubPath::X => Ok((name, Path::TranslateX)),
                            SubPath::Y => Ok((name, Path::TranslateY)),
                        }
                    }
                    "scale" => {
                        let subpath = Self::parse_sub_path(tokens)?;
                        match subpath {
                            SubPath::None => Ok((name, Path::Scale)),
                            SubPath::X => Ok((name, Path::ScaleX)),
                            SubPath::Y => Ok((name, Path::ScaleY)),
                        }
                    }
                    "origin" => {
                        let subpath = Self::parse_sub_path(tokens)?;
                        match subpath {
                            SubPath::None => Ok((name, Path::Origin)),
                            SubPath::X => Ok((name, Path::OriginX)),
                            SubPath::Y => Ok((name, Path::OriginY)),
                        }
                    }
                    "rotation" => Ok((name, Path::Rotate)),
                    _ => Err(format!("Unknown transformation '{trans}'!")),
                };
            } else {
                tokens.putback(next);
                return Err("What transformation do you want to modify?".to_string());
            }
        }
        Err("No part specified!".to_string())
    }

    fn parse_sub_path(tokens: &mut TokenStream) -> Result<SubPath, String> {
        let next = tokens
            .next()
            .ok_or("Unexpected end of stream!".to_string())?;
        if let Token::Dot = next {
            let name = tokens.expect_next_ident()?;
            match name.as_str() {
                "x" => Ok(SubPath::X),
                "y" => Ok(SubPath::Y),
                _ => Err(format!("Unknown transform variant '{:?}'!", name)),
            }
        } else {
            tokens.putback(next);
            Ok(SubPath::None)
        }
    }
}

#[derive(Debug)]
pub struct ParsedKeyframe {
    pub(crate) target: String,
    pub(crate) path: Path,
    pub(crate) value: PathValue,
}

impl ParsedKeyframe {
    pub fn from_num(path: Path, f: f32, target: String) -> Self {
        Self {
            target,
            path,
            value: PathValue::Number(f),
        }
    }

    pub fn from_vec(path: Path, v: Vec2, target: String) -> Self {
        Self {
            target,
            path,
            value: PathValue::Vec2(v),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PathValue {
    Number(f32),
    Vec2(Vec2),
}

impl PathValue {
    pub fn as_f32(&self) -> f32 {
        match self {
            PathValue::Number(f) => *f,
            _ => unimplemented!(),
        }
    }

    pub fn as_vec2(&self) -> Vec2 {
        match self {
            PathValue::Vec2(f) => *f,
            _ => unimplemented!(),
        }
    }
}

enum SubPath {
    None,
    X,
    Y,
}

#[derive(Copy, Clone, Debug)]
pub enum Path {
    Translate,
    TranslateX,
    TranslateY,
    Scale,
    ScaleX,
    ScaleY,
    Rotate,
    Origin,
    OriginX,
    OriginY,
}

#[derive(Debug)]
pub enum Token {
    Parts,
    Bounds,
    Constraints,
    Animations,
    Ident(String),
    Equals,
    Number(f32),
    New(String),
    Keyframe(f32),
    Pre,
    Keyframes,
    Vec2(Vec2),
    Dot,
    Finish,
    Arrow,
}

impl Token {
    pub fn ord(&self) -> u8 {
        let ptr_to_option = (self as *const Token) as *const u8;
        unsafe { *ptr_to_option }
    }
}

struct TokenStream<'a> {
    chars: Peekable<Chars<'a>>,
    putback: VecDeque<Token>,
}

impl<'a> TokenStream<'a> {
    pub fn tokenize(shape_expr: &'a str) -> Self {
        Self {
            chars: shape_expr.chars().peekable(),
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
        if self
            .chars
            .peek()
            .is_some_and(|c| !Self::char_str_valid(*c, allow_numbers))
        {
            return s;
        }
        while let Some(next) = self.chars.next() {
            s.push(next);
            if self
                .chars
                .peek()
                .is_some_and(|c| !Self::char_str_valid(*c, allow_numbers))
            {
                return s;
            }
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

    fn parse_number(&mut self, start: Option<char>) -> Option<f32> {
        let mut num_str = String::new();
        if start.is_some() {
            num_str.push(start.unwrap());
        }
        if self
            .chars
            .peek()
            .is_some_and(|c| !c.is_numeric() && *c != '.' && *c != '-')
        {
            return num_str.parse::<f32>().ok();
        }
        while let Some(next) = self.chars.next() {
            num_str.push(next);
            if self
                .chars
                .peek()
                .is_some_and(|c| !c.is_numeric() && *c != '.' && *c != '-')
            {
                return num_str.parse::<f32>().ok();
            }
        }
        num_str.parse::<f32>().ok()
    }

    pub fn expect_next_ident(&mut self) -> Result<String, String> {
        if let Some(next) = self.next() {
            if let Token::Ident(ident) = next {
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

impl<'a> Iterator for TokenStream<'a> {
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
            if n.is_none() {
                return None;
            }
            next = n.unwrap();
        }
        if next == ';' {
            while next != '\n' {
                let n = self.chars.next();
                if n.is_none() {
                    return None;
                }
                next = n.unwrap();
            }
            while next.is_whitespace() {
                let n = self.chars.next();
                if n.is_none() {
                    return None;
                }
                next = n.unwrap();
            }
        }
        match next {
            '=' => Some(Token::Equals),
            '.' => Some(Token::Dot),
            '<' => Some(Token::Finish),
            '-' => {
                if let Some(n) = self.chars.peek() {
                    if *n == '>' {
                        self.chars.next();
                        return Some(Token::Arrow);
                    }
                }
                self.parse_ident(next)
            }
            '>' => {
                let pc = self.parse_number(None)?;
                Some(Token::Keyframe(pc))
            }
            '#' => {
                let name = self.parse_next_str(None, true);
                match name.as_str() {
                    "PARTS" => Some(Token::Parts),
                    "BOUNDS" => Some(Token::Bounds),
                    "ANIMATIONS" => Some(Token::Animations),
                    "CONSTRAINTS" => Some(Token::Constraints),
                    _ => None,
                }
            }
            '+' => {
                let name = self.parse_next_str(None, true);
                Some(Token::New(name))
            }
            ':' => {
                let sel = self.parse_next_str(None, true);
                match sel.as_str() {
                    "pre" => Some(Token::Pre),
                    "keyframes" => Some(Token::Keyframes),
                    _ => None,
                }
            }
            '[' => {
                let num1 = self.parse_number(None)?;
                let mut next = self.chars.next()?;
                while next.is_whitespace() {
                    let n = self.chars.next();
                    if n.is_none() {
                        return None;
                    }
                    next = n.unwrap();
                }
                if next != ',' {
                    return None;
                }
                let mut next = self.chars.next()?;
                while next.is_whitespace() {
                    let n = self.chars.next();
                    if n.is_none() {
                        return None;
                    }
                    next = n.unwrap();
                }
                let num2 = self.parse_number(Some(next))?;
                let mut next = self.chars.next()?;
                while next.is_whitespace() {
                    let n = self.chars.next();
                    if n.is_none() {
                        return None;
                    }
                    next = n.unwrap();
                }
                if next != ']' {
                    return None;
                }
                Some(Token::Vec2(Vec2::new(num1, num2)))
            }
            _ => self.parse_ident(next),
        }
    }
}

impl<'a> TokenStream<'a> {
    fn parse_ident(&mut self, next: char) -> Option<Token> {
        if next.is_numeric() || next == '-' {
            let lit = self.parse_number(Some(next));
            return if lit.is_some() {
                Some(Token::Number(lit.unwrap()))
            } else {
                None
            };
        }
        let s = self.parse_next_str(Some(next), true);
        Some(Token::Ident(s))
    }
}
