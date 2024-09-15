use std::fmt;
use mvutils::{enum_val, enum_val_ref};

pub struct Entity {
    name: String,
    prefix: Option<String>,
    attributes: Vec<Attribute>,
    inner: Option<XmlValue>
}

enum XmlValue {
    Str(String),
    Entities(Vec<Entity>),
    Code(String)
}

pub struct Attribute {
    name: String,
    value: XmlValue
}

pub fn parse_rsx(input: String) -> Result<Entity, String> {
    let mut lexer = XmlLexer::new(input);
    let en = parse_entity(&mut lexer)?;
    Ok(en)
}

fn parse_entity(lexer: &mut XmlLexer) -> Result<Entity, String> {
    lexer.expect_next(XmlTokenType::LeftAngleBracket)?;
    let tkn = lexer.expect_next(XmlTokenType::Str)?;
    let name = enum_val_ref!(XmlToken, tkn, Str).clone();
    let mut tkn = lexer.next()?;

    let mut attribs = Vec::new();
    while tkn.ordinal() == XmlTokenType::Str.ordinal() {
        let attrib_name = enum_val_ref!(XmlToken, tkn, Str).clone();
        lexer.expect_next(XmlTokenType::Equals)?;
        let type_tkn = lexer.next()?;

        let is_code = type_tkn.ordinal() == XmlTokenType::LeftBrace.ordinal();

        let inner = if is_code {
            let attrib_body = lexer.next()?;
            lexer.expect_next(XmlTokenType::RightBrace)?;
            XmlValue::Code(enum_val!(XmlToken, attrib_body, Code))
        } else {
            XmlValue::Str(enum_val!(XmlToken, type_tkn, Str))
        };

        let attrib = Attribute {
            name: attrib_name,
            value: inner,
        };
        attribs.push(attrib);

        tkn = lexer.next()?;
        println!("{:?}", tkn);
        if tkn.ordinal() != XmlTokenType::Str.ordinal() {
            lexer.putback(tkn.clone());
        }
    }

    println!("1");

    let tkn = lexer.next()?;
    if tkn.ordinal() == XmlTokenType::Slash.ordinal() {
        lexer.expect_next(XmlTokenType::RightAngleBracket)?;

        return Ok(Entity {
            name,
            prefix: None,
            attributes: attribs,
            inner: None,
        });
    } else {
        let mut children = Vec::new();

        let tkn = lexer.next()?;
        if tkn.ordinal() == XmlTokenType::LeftAngleBracket.ordinal() {
            lexer.putback(tkn);
            let mut tkn = lexer.next()?;

            while tkn.ordinal() == XmlTokenType::LeftAngleBracket.ordinal() {
                let next_tkn = lexer.next()?;
                if next_tkn.ordinal() == XmlTokenType::Slash.ordinal() {
                    lexer.putback(next_tkn);
                    lexer.putback(tkn);
                    let entity = Entity {
                        name,
                        prefix: None,
                        attributes: attribs,
                        inner: Some(XmlValue::Entities(children)),
                    };

                    let en = validate_entity_end(lexer, entity)?;
                    return Ok(en);
                }

                lexer.putback(next_tkn);
                lexer.putback(tkn);
                let child = parse_entity(lexer)?;
                children.push(child);
                tkn = lexer.next()?;
            }

        } else if tkn.ordinal() == XmlTokenType::LeftBrace.ordinal() {
            let code_tkn = lexer.expect_next(XmlTokenType::Code)?;
            let code = enum_val!(XmlToken, code_tkn, Code);
            lexer.expect_next(XmlTokenType::RightBrace)?;
            let entity = Entity {
                name,
                prefix: None,
                attributes: attribs,
                inner: Some(XmlValue::Code(code)),
            };
            let en = validate_entity_end(lexer, entity)?;
            return Ok(en);

        } else if tkn.ordinal() == XmlTokenType::Str.ordinal() {
            let str = enum_val!(XmlToken, tkn, Str);

            let entity = Entity {
                name,
                prefix: None,
                attributes: attribs,
                inner: Some(XmlValue::Str(str)),
            };
            let en = validate_entity_end(lexer, entity)?;
            return Ok(en);
        }
    }

    Err("Smth went wrong :(".to_string())
}

fn validate_entity_end(lexer: &mut XmlLexer, en: Entity) -> Result<Entity, String> {
    let en_name = en.name.clone();
    lexer.expect_next(XmlTokenType::LeftAngleBracket)?;
    lexer.expect_next(XmlTokenType::Slash)?;
    let name_tkn = lexer.expect_next(XmlTokenType::Str)?;
    let name = enum_val!(XmlToken, name_tkn, Str);
    if name != en_name {
        return Err("Opening and closing tags must match!".to_string());
    }
    lexer.expect_next(XmlTokenType::RightAngleBracket)?;
    Ok(en)
}

#[repr(u8)]
#[derive(Debug, Clone)]
enum XmlToken {
    LeftAngleBracket,
    RightAngleBracket,
    Str(String),
    Code(String),
    Equals,
    LeftBrace,
    RightBrace,
    Slash,
}

impl XmlToken {
    fn ordinal(&self) -> u8 {
        unsafe {
            *(self as *const XmlToken as *const u8)
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone)]
enum XmlTokenType {
    LeftAngleBracket,
    RightAngleBracket,
    Str,
    Code,
    Equals,
    LeftBrace,
    RightBrace,
    Slash,
}

impl XmlTokenType {
    fn ordinal(&self) -> u8 {
        unsafe {
            *(self as *const XmlTokenType as *const u8)
        }
    }
}

struct XmlLexer {
    input: String,
    idx: usize,
    in_code_block: bool,
    putback: Vec<XmlToken>
}

impl XmlLexer {
    pub fn new(input: String) -> Self {
        Self {
            input,
            idx: 0,
            in_code_block: false,
            putback: vec![],
        }
    }

    fn c(&self) -> Result<char, String> {
        self.input.chars().nth(self.idx).ok_or(format!("expected char at position {}", self.idx + 1).to_string())
    }

    //fn peek(&self) -> Option<char> {
    //    self.input.chars().nth(self.idx + 1)
    //}

    pub fn putback(&mut self, tkn: XmlToken) {
        self.putback.push(tkn);
    }

    pub fn expect_next(&mut self, ty: XmlTokenType) -> Result<XmlToken, String> {
        let next = self.next()?;
        if next.ordinal() == ty.ordinal() {
            return Ok(next);
        }
        Err(format!("Expected {:?}, got {:?} at position {}", ty, next, self.idx).to_string())
    }

    pub fn next(&mut self) -> Result<XmlToken, String> {
        if !self.putback.is_empty() {
            return self.putback.pop().ok_or("Idk whats wrong but this line is shorter :P".to_string());
        }

        if self.in_code_block {
            let mut code = String::new();
            let mut brace_count = 1;
            loop {
                let n = self.c()?;
                self.idx += 1;
                if n == '{' {
                    brace_count += 1;
                } else if n == '}' {
                    brace_count -= 1;
                    if brace_count == 0 {
                        self.idx -= 1;
                        self.in_code_block = false;
                        return Ok(XmlToken::Code(code));
                    }
                }
                code.push(n);
            }
        }

        let mut next = self.c()?;
        self.idx += 1;
        if next.is_whitespace() {
            next = self.c()?;
            self.idx += 1;
        }

        match next {
            '<' => Ok(XmlToken::LeftAngleBracket),
            '>' => Ok(XmlToken::RightAngleBracket),
            '/' => Ok(XmlToken::Slash),
            '=' => Ok(XmlToken::Equals),
            '"' => {
                let mut str = String::new();
                loop {
                    let n = self.c()?;
                    self.idx += 1;
                    if n == '\\' {
                        let n = self.c()?;
                        self.idx += 1;
                        str.push(n);
                    } else {
                        if n == '"' {
                            //self.idx += 1;
                            return Ok(XmlToken::Str(str));
                        }
                        str.push(n);
                    }
                }
            }
            '{' => {
                self.in_code_block = true;
                Ok(XmlToken::LeftBrace)
            }
            '}' => Ok(XmlToken::RightBrace),
            _ => {
                let mut str = String::new();
                self.idx -= 1;
                str.push(self.c()?);
                self.idx += 1;
                loop {
                    let n = self.c()?;
                    self.idx += 1;
                    if !n.is_alphanumeric() && n != '_' {
                        self.idx -= 1;
                        return Ok(XmlToken::Str(str));
                    }
                    str.push(n);
                }
            }
        }
    }
}

//
//  Debug
//

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entity")
            .field("name", &self.name)
            .field("prefix", &self.prefix)
            .field("attributes", &self.attributes)
            .field("inner", &self.inner)
            .finish()
    }
}

impl fmt::Debug for XmlValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XmlValue::Str(s) => write!(f, "Str({:?})", s),
            XmlValue::Entities(entities) => {
                write!(f, "Entities({:#?})", entities) // Pretty-print for nested entities
            },
            XmlValue::Code(code) => write!(f, "Code({:?})", code),
        }
    }
}

impl fmt::Debug for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Attribute")
            .field("name", &self.name)
            .field("value", &self.value)
            .finish()
    }
}