use crate::rendering::loading::ModelLoadingError;
use mvutils::TryFromString;
use mvutils::once::Lazy;
use regex::Regex;
use std::collections::VecDeque;
use std::iter::Peekable;
use std::str::{Chars, FromStr};

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Token {
    Command(Command),
    StrLit(String),
    FloatLit(f32),
}

impl Token {
    pub fn want_command(self) -> Result<Command, ModelLoadingError> {
        match self {
            Token::Command(c) => Ok(c),
            other => Err(ModelLoadingError::IllegalContent(format!(
                "Expected Command, found {other:?}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, TryFromString)]
pub(super) enum Command {
    // ---------- .obj ----------
    #[casing(Lower)]
    O, // o <object name>
    #[casing(Lower)]
    G, // g <group name>
    #[casing(Lower)]
    V, // v x y z
    #[casing(Lower)]
    VT, // vt u v [w]
    #[casing(Lower)]
    VN, // vn x y z
    #[casing(Lower)]
    F, // f v1[/vt1/vn1] ...
    #[casing(Lower)]
    Usemtl, // usemtl <name>
    #[casing(Lower)]
    Mtllib, // mtllib <file>
    #[casing(Lower)]
    S, // s <group smoothing>

    // ---------- .mtl ----------
    #[pattern("(?i)^newmtl$")]
    Newmtl, // newmtl <name>
    #[pattern("(?i)^ka$")]
    Ka,
    #[pattern("(?i)^kd$")]
    Kd,
    #[pattern("(?i)^ks$")]
    Ks,
    #[pattern("(?i)^ke$")]
    Ke,
    #[pattern("(?i)^ns$")]
    Ns,
    #[pattern("(?i)^ni$")]
    Ni,
    #[pattern("(?i)^d$")]
    D,
    #[pattern("(?i)^tr$")]
    Tr,
    #[pattern("(?i)^illum$")]
    Illum,

    // ---------- texture maps ----------
    #[pattern("(?i)^map_?ka$")]
    MapKa,
    #[pattern("(?i)^map_?kd$")]
    MapKd,
    #[pattern("(?i)^map_?ks$")]
    MapKs,
    #[pattern("(?i)^(map_?bump|bump)$")]
    MapBump,
}

pub(super) struct Tokenizer<'a> {
    source: Peekable<Chars<'a>>,
    putback: VecDeque<Token>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.chars().peekable(),
            putback: VecDeque::new(),
        }
    }

    pub fn putback(&mut self, token: Token) {
        self.putback.push_back(token);
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.source.peek() {
            if c.is_whitespace() {
                self.source.next();
            } else if c == '#' {
                while let Some(ch) = self.source.next() {
                    if ch == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    fn read_word(&mut self, first: char) -> String {
        let mut s = String::new();
        s.push(first);
        while let Some(&ch) = self.source.peek() {
            if ch.is_whitespace() {
                break;
            }
            s.push(ch);
            self.source.next();
        }
        s
    }

    fn read_number(&mut self, first: char) -> String {
        let mut s = String::new();
        s.push(first);
        while let Some(&c) = self.source.peek() {
            if c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '-' || c == '+' {
                s.push(c);
                self.source.next();
            } else {
                break;
            }
        }
        s
    }

    fn word_to_command(word: &str) -> Option<Command> {
        use Command::*;
        Some(match word {
            "o" => O,
            "g" => G,
            "v" => V,
            "vt" => VT,
            "vn" => VN,
            "f" => F,
            "usemtl" => Usemtl,
            "mtllib" => Mtllib,
            "s" => S,
            "newmtl" => Newmtl,
            "Ka" => Ka,
            "Kd" => Kd,
            "Ks" => Ks,
            "Ke" => Ke,
            "Ns" => Ns,
            "Ni" => Ni,
            "d" => D,
            "Tr" => Tr,
            "illum" => Illum,
            "map_Kd" => MapKd,
            "map_Ka" => MapKa,
            "map_Ks" => MapKs,
            "map_Bump" | "bump" => MapBump,
            _ => return None,
        })
    }

    pub(super) fn next_command(&mut self) -> Result<Command, ModelLoadingError> {
        let token = self.next().ok_or(ModelLoadingError::UnexpectedEndOfFile)?;
        token.want_command()
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(tok) = self.putback.pop_front() {
            return Some(tok);
        }

        self.skip_whitespace();
        let c = self.source.next()?;

        // ---- Commands and identifiers ----
        if c.is_ascii_alphabetic() {
            let word = self.read_word(c);

            return if let Ok(cmd) = Command::from_str(word.as_str()) {
                Some(Token::Command(cmd))
            } else {
                Some(Token::StrLit(word))
            };
        }

        // ---- Numbers ----
        if c.is_ascii_digit() || c == '-' || c == '+' {
            let mut num_str = self.read_number(c);
            while let Some('/') = self.source.peek() {
                self.source.next();
                let next = self.source.next()?;
                let num = self.read_number(next);
                num_str = format!("{num_str}/{num}");
            }
            if let Ok(f) = num_str.parse::<f32>() {
                return Some(Token::FloatLit(f));
            }
            return Some(Token::StrLit(num_str));
        }

        if !c.is_whitespace() {
            return Some(Token::StrLit(c.to_string()));
        }

        None
    }
}
