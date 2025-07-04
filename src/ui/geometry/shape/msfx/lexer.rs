use crate::ui::parse;
use mvutils::TryFromString;
use std::collections::VecDeque;
use std::iter::Peekable;
use std::num::ParseFloatError;
use std::str::{Chars, FromStr};

#[derive(TryFromString, Debug)]
pub enum MSFXKeyword {
    If,
    Else,
    Let,
    For,
    While,
    Break,
    Continue,
    Export,
    Input,
    Bool,
    Number,
    Vec2,
    In,
    End,
    Adaptive,
    Is,
    Isnt,
    And,
    Or,
    Begin,
}

#[derive(Debug, Clone)]
pub enum MSFXOperator {
    Dot,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    And,
    Or,
    Not,
    Assign,
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
}

impl MSFXOperator {
    pub fn is_unary(&self) -> bool {
        match self {
            MSFXOperator::Not | MSFXOperator::Sub => true,
            _ => false,
        }
    }

    pub fn precedence(&self) -> u8 {
        match self {
            MSFXOperator::Dot => 7,
            MSFXOperator::Not => 6,
            MSFXOperator::Pow => 5,
            MSFXOperator::Mul | MSFXOperator::Div | MSFXOperator::Mod => 4,
            MSFXOperator::Add | MSFXOperator::Sub => 3,
            MSFXOperator::Lt | MSFXOperator::Gt | MSFXOperator::Lte | MSFXOperator::Gte => 2,
            MSFXOperator::Eq | MSFXOperator::Neq => 1,
            MSFXOperator::And | MSFXOperator::Or | MSFXOperator::Assign => 0,
        }
    }
}

#[derive(Debug)]
pub enum MSFXToken {
    Comma,
    Colon,
    Semicolon,
    LBrack,
    RBrack,
    LParen,
    RParen,
    Hashtag,

    Keyword(MSFXKeyword),
    Operator(MSFXOperator),
    OperatorAssign(MSFXOperator),
    Ident(String),
    Literal(f64),

    Error(String),
    EOF,
}

impl MSFXToken {
    pub fn ord(&self) -> u8 {
        let ptr_to_option = (self as *const MSFXToken) as *const u8;
        unsafe { *ptr_to_option }
    }

    pub fn to_ident(self) -> Result<String, String> {
        match self {
            Self::Ident(i) => Ok(i),
            _ => Err(format!("Expected Ident, found: {self:?}")),
        }
    }

    pub fn op(&self) -> Option<MSFXOperator> {
        match self {
            Self::Operator(o) => Some(o.clone()),
            Self::OperatorAssign(o) => Some(o.clone()),
            _ => None,
        }
    }

    pub fn assign(&self) -> bool {
        match self {
            Self::OperatorAssign(_) => true,
            _ => false,
        }
    }
}

pub struct MSFXLexer<'a> {
    chars: Peekable<Chars<'a>>,
    putback: VecDeque<MSFXToken>,
}

impl<'a> MSFXLexer<'a> {
    pub fn lex(expr: &'a str) -> Self {
        Self {
            chars: expr.chars().peekable(),
            putback: VecDeque::new(),
        }
    }

    pub fn putback(&mut self, token: MSFXToken) {
        self.putback.push_back(token);
    }

    pub fn next_ident(&mut self) -> Result<String, String> {
        let t = self.next();
        match t {
            MSFXToken::EOF => Err("Unexpected EOF, expected Ident".to_string()),
            MSFXToken::Error(s) => Err(s),
            t => {
                if let MSFXToken::Ident(ident) = t {
                    Ok(ident)
                } else {
                    Err(format!("Unexpected Token, expected Ident, got: {t:?}"))
                }
            }
        }
    }

    pub fn next_token(&mut self, tkn: MSFXToken) -> Result<(), String> {
        let t = self.next();
        if t.ord() == tkn.ord() {
            Ok(())
        } else {
            Err(format!("Unexpected Token, expected {tkn:?}, got: {t:?}"))
        }
    }

    pub fn next_some(&mut self) -> Result<MSFXToken, String> {
        let t = self.next();
        if let MSFXToken::EOF = &t {
            Err("Unexpected EOF, expected Some".to_string())
        } else if let MSFXToken::Error(e) = &t {
            Err(e.clone())
        } else {
            Ok(t)
        }
    }

    fn collect_until<P: Fn(char) -> bool>(&mut self, predicate: P) -> String {
        self.collect_until_with(String::new(), predicate)
    }

    fn collect_until_with<P: Fn(char) -> bool>(&mut self, mut s: String, predicate: P) -> String {
        let mut next = self.chars.peek().cloned();
        while let Some(n) = next {
            if predicate(n) {
                return s;
            }
            s.push(n);
            self.chars.next();
            next = self.chars.peek().cloned();
        }
        s
    }

    fn collect_until_newline(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c == '\n' {
                self.chars.next();
                break;
            } else if c == '\r' {
                self.chars.next();
                if let Some(&'\n') = self.chars.peek() {
                    self.chars.next();
                }
                break;
            } else {
                self.chars.next();
            }
        }
    }

    pub fn next(&mut self) -> MSFXToken {
        if !self.putback.is_empty() {
            return self.putback.pop_back().unwrap();
        }

        let _ = self.collect_until(|x| !x.is_whitespace());
        let next = self.chars.next();

        if let Some('/') = next {
            match self.chars.peek() {
                Some(&'/') => {
                    self.chars.next();
                    self.collect_until_newline();
                    return self.next();
                }
                _ => {}
            }
        }

        macro_rules! potentially_assign {
            ($op:ident) => {{
                if let Some(&'=') = self.chars.peek() {
                    self.chars.next();
                    return MSFXToken::OperatorAssign(MSFXOperator::$op);
                }
                return MSFXToken::Operator(MSFXOperator::$op);
            }};
        }

        if let Some(n) = next {
            match n {
                d if d.is_numeric() => {
                    let data = self.collect_until_with(d.to_string(), |c| {
                        !(c.is_numeric() || c == '.' || c == 'e' || c == '_')
                    });
                    return parse::parse_num::<f64, ParseFloatError>(&data)
                        .map(MSFXToken::Literal)
                        .unwrap_or_else(MSFXToken::Error);
                }
                '#' => return MSFXToken::Hashtag,
                'π' => return MSFXToken::Literal(std::f64::consts::PI),
                '>' => {
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        return MSFXToken::Operator(MSFXOperator::Gte);
                    }
                    return MSFXToken::Operator(MSFXOperator::Gt);
                }
                '<' => {
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        return MSFXToken::Operator(MSFXOperator::Lte);
                    }
                    return MSFXToken::Operator(MSFXOperator::Lt);
                }
                '=' => {
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        return MSFXToken::Operator(MSFXOperator::Eq);
                    }
                    return MSFXToken::Operator(MSFXOperator::Assign);
                }

                ',' => return MSFXToken::Comma,
                ';' => return MSFXToken::Semicolon,
                '[' => return MSFXToken::LBrack,
                ']' => return MSFXToken::RBrack,
                '(' => return MSFXToken::LParen,
                ')' => return MSFXToken::RParen,
                '.' => return MSFXToken::Operator(MSFXOperator::Dot),
                ':' => return MSFXToken::Colon,

                '+' => potentially_assign!(Add),
                '-' => potentially_assign!(Sub),
                '*' => potentially_assign!(Mul),
                '/' => potentially_assign!(Div),
                '%' => potentially_assign!(Mod),
                '^' => potentially_assign!(Pow),

                '!' => {
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        return MSFXToken::Operator(MSFXOperator::Neq);
                    }
                    return MSFXToken::Operator(MSFXOperator::Not);
                }

                _ => {
                    const ALLOWED_SPECIAL: [char; 14] = [
                        '_', '\'', '{', '}', '?', '$', '\\', '€', '@', '|', '&', '~', '😱', '🚨',
                    ];
                    let s = self.collect_until_with(n.to_string(), |x| {
                        !(x.is_alphanumeric()
                            || x == '_'
                            || x == '\''
                            || ALLOWED_SPECIAL.contains(&x))
                    });
                    if let Ok(keyword) = MSFXKeyword::from_str(&s) {
                        macro_rules! keyword_op {
                            ($keyword_val:ident, $op_val: ident) => {
                                if let MSFXKeyword::$keyword_val = keyword {
                                    return MSFXToken::Operator(MSFXOperator::$op_val);
                                }
                            };
                        }
                        keyword_op!(Is, Eq);
                        keyword_op!(Isnt, Neq);
                        keyword_op!(And, And);
                        keyword_op!(Or, Or);
                        return MSFXToken::Keyword(keyword);
                    } else {
                        if s.to_lowercase() == "isn't" {
                            return MSFXToken::Operator(MSFXOperator::Neq);
                        } else if s.to_lowercase() == "null" {
                            return MSFXToken::Hashtag;
                        }
                        return MSFXToken::Ident(s);
                    }
                }
            }
        }

        MSFXToken::EOF
    }
}
