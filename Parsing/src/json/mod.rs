pub mod from_json;
pub mod types;

use crate::json::types::{JsonArray, JsonNumber, JsonObject};
use std::collections::VecDeque;
use std::str::FromStr;
use types::JsonElement;

#[derive(Copy, Clone)]
pub enum JsonIdentFlavor {
    UnquotedIdents,
    QuotedIdentifiers,
    Identifiers,
}

impl JsonIdentFlavor {
    fn ident<I: Iterator<Item = char>>(
        &self,
        lexer: &mut StringLexer<I>,
    ) -> Result<String, String> {
        lexer.skip_whitespace();
        match self {
            JsonIdentFlavor::UnquotedIdents => {
                let name = lexer.collect_exclusive(':');
                Ok(name)
            }
            JsonIdentFlavor::QuotedIdentifiers => {
                lexer.pop_string('"')?;
                let name = lexer.collect_exclusive(':');
                lexer.pop_string('"')?;
                Ok(name)
            }
            JsonIdentFlavor::Identifiers => {
                if lexer.has_next_char('"') {
                    lexer.pop_string('"')?;
                    let name = lexer.collect_exclusive(':');
                    lexer.pop_string('"')?;
                    Ok(name)
                } else {
                    let name = lexer.collect_exclusive(':');
                    Ok(name)
                }
            }
        }
    }
}

pub fn parse_json(json: &str, flavor: JsonIdentFlavor) -> Result<JsonElement, String> {
    let mut lexer = StringLexer::new(json.chars());
    parse_element(&mut lexer, flavor)
}

fn parse_element<I: Iterator<Item = char>>(
    lexer: &mut StringLexer<I>,
    flavor: JsonIdentFlavor,
) -> Result<JsonElement, String> {
    lexer.skip_whitespace();
    if lexer.has_next_char('"') {
        lexer.pop_string('"')?;
        let s = lexer.collect_exclusive('"');
        lexer.pop_string('"')?;
        Ok(JsonElement::Str(s))
    } else if lexer.has_next_char('{') {
        lexer.pop_string('{')?;
        lexer.skip_whitespace();
        let mut obj = JsonObject::new();
        while !lexer.has_next_char('}') {
            let (field_name, field_value) = parse_field(lexer, flavor)?;
            lexer.skip_whitespace();
            if lexer.has_next_char(',') {
                lexer.pop_string(',')?;
                lexer.skip_whitespace();
            }
            obj.field(field_name, field_value);
        }
        lexer.pop_string('}')?;
        Ok(JsonElement::Object(obj))
    } else if lexer.has_next_char('[') {
        lexer.pop_string('[')?;
        lexer.skip_whitespace();
        let mut array = JsonArray::new();
        while !lexer.has_next_char(']') {
            let elem = parse_element(lexer, flavor)?;
            lexer.skip_whitespace();
            if lexer.has_next_char(',') {
                lexer.pop_string(',')?;
                lexer.skip_whitespace();
            }
            array.push(elem);
        }
        lexer.pop_string(']')?;
        Ok(JsonElement::Array(array))
    } else if lexer.has_next() {
        lexer.skip_whitespace();
        let n_str =
            lexer.collect_while(|c| c.is_ascii_digit() || ['-', '+', '.', 'e', 'E'].contains(&c));
        if let Ok(int) = i32::from_str(&n_str) {
            Ok(JsonElement::Number(JsonNumber::Int(int)))
        } else if let Ok(float) = f32::from_str(&n_str) {
            Ok(JsonElement::Number(JsonNumber::Float(float)))
        } else {
            Err(format!("Cannot parse '{n_str}' as number!"))
        }
    } else {
        Err("Expected any character, found nothing!".to_string())
    }
}

fn parse_field<I: Iterator<Item = char>>(
    lexer: &mut StringLexer<I>,
    flavor: JsonIdentFlavor,
) -> Result<(String, JsonElement), String> {
    let name = flavor.ident(lexer)?;
    lexer.skip_whitespace();
    lexer.pop_string(':')?;
    let element = parse_element(lexer, flavor)?;

    Ok((name, element))
}

pub struct StringLexer<I: Iterator<Item = char>> {
    source: I,
    buffer: VecDeque<char>,
}

impl<I: Iterator<Item = char>> StringLexer<I> {
    pub fn new(source: I) -> Self {
        Self {
            source,
            buffer: VecDeque::new(),
        }
    }

    pub fn skip_whitespace(&mut self) {
        while let Some(c) = self.next_char() {
            if !c.is_whitespace() {
                self.buffer.push_back(c);
                break;
            }
        }
    }

    pub fn next_char(&mut self) -> Option<char> {
        if !self.buffer.is_empty() {
            return self.buffer.pop_front();
        }
        self.source.next()
    }

    pub fn collect_exclusive(&mut self, gate: char) -> String {
        let mut s = String::new();
        while let Some(next) = self.next_char() {
            if next == gate {
                self.buffer.push_back(next);
                break;
            }
            s.push(next);
        }
        s
    }

    pub fn collect_while<F: Fn(char) -> bool>(&mut self, predicate: F) -> String {
        let mut s = String::new();
        while let Some(next) = self.next_char() {
            if !predicate(next) {
                self.buffer.push_back(next);
                break;
            }
            s.push(next);
        }
        s
    }

    pub fn pop<E: From<String>>(&mut self, expected: char) -> Result<(), E> {
        let next = self
            .next_char()
            .ok_or(E::from(format!("Expected {expected}, found nothing!")))?;
        if next == expected {
            return Ok(());
        }
        Err(E::from(format!("Expected {expected}, found {next}!")))
    }

    pub fn pop_string(&mut self, expected: char) -> Result<(), String> {
        self.pop(expected)
    }

    pub fn has_next(&mut self) -> bool {
        if let Some(n) = self.next_char() {
            self.buffer.push_back(n);
            true
        } else {
            false
        }
    }

    pub fn has_next_char(&mut self, c: char) -> bool {
        if let Some(n) = self.next_char() {
            self.buffer.push_back(n);
            n == c
        } else {
            false
        }
    }
}
