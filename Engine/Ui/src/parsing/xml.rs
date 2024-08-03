use crate::ui::parsing::{BytesIter, Parser, Tag};
use parking_lot::RwLock;
use std::borrow::Cow;
use std::iter::Peekable;
use std::slice::Iter;
use std::str::Bytes;
use std::sync::Arc;

pub struct XmlParser {
    lexer: XmlLexer,
}

impl Parser for XmlParser {
    fn next(&self) -> Option<Tag> {
        todo!()
    }

    fn inner(&self) -> Option<Arc<RwLock<dyn Parser>>>
    where
        Self: Sized,
    {
        todo!()
    }

    fn parse(bytes: BytesIter) -> Self
    where
        Self: Sized,
    {
        let iter = bytes.peekable();

        Self {
            lexer: XmlLexer {
                data: iter,
                next: vec![],
            },
        }
    }
}

#[derive(Clone, Debug)]
pub enum XmlToken {
    TagBegin,
    TagEnd,
    NameSpace(String),
    Name(String),
    Attribute(String, String),
    None,
}

pub struct XmlLexer {
    data: Peekable<BytesIter>,
    next: Vec<XmlToken>,
}

impl XmlLexer {
    pub fn new(data: Peekable<BytesIter>) -> Self {
        Self { data, next: vec![] }
    }
}

#[derive(Clone)]
pub struct XmlError {
    pub cause: String,
}

impl XmlError {
    pub fn new(cause: &str) -> Self {
        Self {
            cause: cause.to_owned(),
        }
    }
}

impl Default for XmlError {
    fn default() -> Self {
        Self {
            cause: "unknown error".to_string(),
        }
    }
}

impl Iterator for XmlLexer {
    type Item = Result<XmlToken, XmlError>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.next.is_empty() {
            return Some(Ok(self.next.remove(0)));
        }

        let next = self.data.next();
        if next.is_none() {
            return None;
        }
        let next = next.unwrap();

        if next != b'<' {
            //Invalid XML
            return Some(Err(XmlError::new("Expected Token TagStart")));
        }

        let mut str_buf = String::new();

        let mut next = self.data.next();

        let mut in_attr = false;
        let mut in_val = false;

        loop {
            if next.is_none() {
                return Some(Err(XmlError::new("Unexpected end of XML")));
            }

            let byte = next.unwrap();

            fn get_attrib_tkn(str: String) -> XmlToken {
                let (name, val) = str.split_once("=").unwrap();
                let val = val.strip_prefix("\"").unwrap().strip_suffix("\"").unwrap();

                XmlToken::Attribute(name.to_string(), val.to_string())
            }

            if byte == b':' {
                self.next.push(XmlToken::NameSpace(str_buf.clone()));
                str_buf.clear();
            } else if byte == b' ' {
                if !in_attr {
                    self.next.push(XmlToken::Name(str_buf.clone()));
                    str_buf.clear();
                    in_attr = true
                } else {
                    if !in_val {
                        let ret = get_attrib_tkn(str_buf.clone());
                        str_buf.clear();
                        in_attr = false;
                        self.next.push(ret);
                    }
                }
            } else {
                str_buf.push(byte as char)
            }

            next = self.data.next();
        }

        return Some(Ok(XmlToken::TagBegin));

        None
    }
}
