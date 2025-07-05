use crate::math::vec::Vec2;
use std::collections::VecDeque;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Clone, Debug)]
#[repr(u8)]
pub enum MRFToken {
    Parts,
    Bones,
    Joints,
    Attach,

    Colon,
    Ident(String),
    Vec2(Vec2),
    Anchor,
    Selector,
    Error(String),
    EOF,
}

impl MRFToken {
    pub fn ord(&self) -> u8 {
        let ptr_to_option = (self as *const MRFToken) as *const u8;
        unsafe { *ptr_to_option }
    }
}

pub struct MRFLexer<'a> {
    chars: Peekable<Chars<'a>>,
    putback: VecDeque<MRFToken>,
}

impl<'a> MRFLexer<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            chars: s.chars().peekable(),
            putback: VecDeque::new(),
        }
    }

    pub fn next_ident(&mut self) -> Result<String, String> {
        let t = self.next();
        if let Some(t) = t {
            if let MRFToken::Ident(ident) = t {
                Ok(ident)
            } else {
                Err(format!("Unexpected Token, expected Ident, got: {t:?}"))
            }
        } else {
            Err("Unexpected EOF, expected Ident".to_string())
        }
    }

    pub fn next_vec2(&mut self) -> Result<Vec2, String> {
        let t = self.next();
        if let Some(t) = t {
            if let MRFToken::Vec2(vec2) = t {
                Ok(vec2)
            } else {
                Err(format!("Unexpected Token, expected Vec2, got: {t:?}"))
            }
        } else {
            Err("Unexpected EOF, expected Vec2".to_string())
        }
    }

    pub fn next_token(&mut self, tkn: MRFToken) -> Result<(), String> {
        let t = self.next();
        if let Some(t) = t {
            if t.ord() == tkn.ord() {
                Ok(())
            } else {
                Err(format!("Unexpected Token, expected {tkn:?}, got: {t:?}"))
            }
        } else {
            Err(format!("Unexpected EOF, expected {tkn:?}"))
        }
    }

    pub fn next_some(&mut self) -> Result<MRFToken, String> {
        let t = self.next();
        if let Some(t) = t {
            Ok(t)
        } else {
            Err("Unexpected EOF, expected Some".to_string())
        }
    }

    pub fn putback(&mut self, token: MRFToken) {
        self.putback.push_back(token);
    }

    pub fn next(&mut self) -> Option<MRFToken> {
        if !self.putback.is_empty() {
            return self.putback.pop_front();
        }
        let mut next = self.chars.next();
        // skip whitespaces and comments
        while let Some(c) = next {
            if c.is_whitespace() {
                next = self.chars.next();
            } else if c == '/' {
                let p = self.chars.peek().cloned();
                if let Some(p) = p {
                    if p == '/' {
                        self.chars.next();
                        // skip everything until newline
                        while let Some(&sn) = self.chars.peek() {
                            if sn == '\n' {
                                self.chars.next();
                                break;
                            } else if sn == '\r' {
                                self.chars.next();
                                if let Some(&'\n') = self.chars.peek() {
                                    self.chars.next();
                                }
                                break;
                            } else {
                                self.chars.next();
                            }
                        }
                        next = self.chars.next();
                        continue;
                    }
                }
                break; // just a single '/' that isn't part of a comment
            } else {
                break;
            }
        }

        if let Some(n) = next {
            return match n {
                '#' => {
                    //section
                    let mut name = String::new();
                    let mut n = self.chars.peek().cloned();
                    while let Some(c) = n {
                        if c.is_alphabetic() {
                            self.chars.next();
                            name.push(c);
                            n = self.chars.peek().cloned();
                        } else {
                            break;
                        }
                    }
                    match name.as_str() {
                        "PARTS" => Some(MRFToken::Parts),
                        "BONES" => Some(MRFToken::Bones),
                        "JOINTS" => Some(MRFToken::Joints),
                        "ATTACH" => Some(MRFToken::Attach),
                        _ => Some(MRFToken::Error(format!("'{name}' is not a valid section!"))),
                    }
                }
                ':' => Some(MRFToken::Colon),
                '>' => Some(MRFToken::Selector),
                _ => {
                    //ident or number or vec2
                    let mut s = String::new();
                    s.push(n);
                    let is_ident = n.is_alphabetic();
                    let mut p = self.chars.peek().cloned();
                    while let Some(sp) = p {
                        if sp.is_alphanumeric()
                            || sp == '_'
                            || (!is_ident && (sp == '.' || sp == ',' || sp == '-'))
                        {
                            self.chars.next();
                            s.push(sp);
                            p = self.chars.peek().cloned();
                        } else {
                            break;
                        }
                    }
                    if is_ident {
                        if s == "anchor" {
                            Some(MRFToken::Anchor)
                        } else {
                            Some(MRFToken::Ident(s))
                        }
                    } else {
                        // vec2
                        let lr = s.split_once(',');
                        if let Some(lr) = lr {
                            let (left, right) = lr;
                            let left = left.trim();
                            let right = right.trim();

                            let left_res = left.parse::<f32>();
                            if left_res.is_err() {
                                Some(MRFToken::Error(left_res.unwrap_err().to_string()))
                            } else {
                                let left = left_res.unwrap();

                                let right_res = right.parse::<f32>();
                                return if right_res.is_err() {
                                    Some(MRFToken::Error(right_res.unwrap_err().to_string()))
                                } else {
                                    let right = right_res.unwrap();
                                    return Some(MRFToken::Vec2(Vec2::new(left, right)));
                                };
                            }
                        } else {
                            Some(MRFToken::Error(format!("'{s}' is not a proper vec2!")))
                        }
                    }
                }
            };
        }

        Some(MRFToken::EOF)
    }
}
