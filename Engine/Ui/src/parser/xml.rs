use std::fmt;
use std::io::empty;
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
    parse_entity(&mut lexer)
}

fn parse_entity(lexer: &mut XmlLexer) -> Result<Entity, String> {
    lexer.expect_next(XmlTokenType::LeftAngleBracket)?;
    let name = lexer.expect_next(XmlTokenType::Literal)?.as_literal();

    let mut tkn = lexer.next()?;

    //
    // Attributes
    //

    let mut attributes = Vec::new();
    if tkn.is(XmlTokenType::Literal) {
        while tkn.is(XmlTokenType::Literal) {
            let attrib_name = tkn.as_literal();
            lexer.expect_next(XmlTokenType::Equals)?;
            let next_tkn = lexer.next()?;

            let inner = if next_tkn.is(XmlTokenType::Quote) {
                let str_tkn = lexer.expect_next(XmlTokenType::Literal)?;
                lexer.expect_next(XmlTokenType::Quote)?;
                XmlValue::Str(str_tkn.as_literal())
            } else {
                let code_tkn = lexer.expect_next(XmlTokenType::Code)?;
                lexer.expect_next(XmlTokenType::RightBrace)?;
                XmlValue::Code(code_tkn.as_code())
            };

            let attrib = Attribute {
                name: attrib_name,
                value: inner,
            };
            attributes.push(attrib);

            tkn = lexer.next()?;
            if !tkn.is(XmlTokenType::Literal) {
                lexer.putback(tkn.clone());
            }
        }
    } else {
        lexer.putback(tkn);
    }

    let tkn = lexer.next()?;
    match tkn {
        XmlToken::Slash => {
            lexer.expect_next(XmlTokenType::RightAngleBracket)?;
            Ok(Entity {
                name,
                prefix: None,
                attributes,
                inner: None,
            })
        }
        XmlToken::RightAngleBracket => {
            let mut tkn = lexer.next()?;
            if tkn.is(XmlTokenType::Literal) {
                let inner = XmlValue::Str(tkn.as_literal());
                let entity = Entity {
                    name,
                    prefix: None,
                    attributes,
                    inner: Some(inner),
                };
                validate_entity(lexer, entity)
            } else if tkn.is(XmlTokenType::LeftBrace) {
                let tkn = lexer.expect_next(XmlTokenType::Code)?;
                let inner = XmlValue::Code(tkn.as_code());
                lexer.expect_next(XmlTokenType::RightBrace)?;
                let entity = Entity {
                    name,
                    prefix: None,
                    attributes,
                    inner: Some(inner),
                };
                validate_entity(lexer, entity)
            } else if tkn.is(XmlTokenType::LeftAngleBracket) {
                let next_tkn = lexer.next()?;
                if next_tkn.is(XmlTokenType::Slash) {
                    lexer.putback(next_tkn);
                    lexer.putback(tkn);
                    let entity = Entity {
                        name,
                        prefix: None,
                        attributes,
                        inner: None,
                    };
                    validate_entity(lexer, entity)
                } else {
                    let mut entities = Vec::new();

                    lexer.putback(next_tkn);

                    while tkn.is(XmlTokenType::LeftAngleBracket) {
                        let next_tkn = lexer.next()?;
                        if next_tkn.is(XmlTokenType::Slash) {
                            lexer.putback(next_tkn);
                            lexer.putback(tkn.clone());
                            break;
                        }

                        lexer.putback(next_tkn);
                        lexer.putback(tkn.clone());

                        let inner = parse_entity(lexer)?;
                        entities.push(inner);

                        tkn = lexer.next()?;
                    }

                    let entity = Entity {
                        name,
                        prefix: None,
                        attributes,
                        inner: Some(XmlValue::Entities(entities)),
                    };
                    validate_entity(lexer, entity)
                }
            } else {
                Err(format!("Unexpected token1, got {:?}", tkn).to_string())
            }
        }
        _ => {
            Err(format!("Unexpected token, got {:?}", tkn).to_string())
        }
    }
}

fn validate_entity(lexer: &mut XmlLexer, en: Entity) -> Result<Entity, String> {
    lexer.expect_next(XmlTokenType::LeftAngleBracket)?;
    lexer.expect_next(XmlTokenType::Slash)?;
    let name = lexer.expect_next(XmlTokenType::Literal)?.as_literal();
    if name == en.name {
        lexer.expect_next(XmlTokenType::RightAngleBracket)?;
        Ok(en)
    } else {
        Err("Closing tag must match opening tag name".to_string())
    }
}

#[repr(u8)]
#[derive(Debug, Clone)]
enum XmlToken {
    LeftAngleBracket,
    RightAngleBracket,
    Literal(String),
    Code(String),
    Equals,
    LeftBrace,
    RightBrace,
    Slash,
    Quote,
}

impl XmlToken {
    fn ordinal(&self) -> u8 {
        unsafe {
            *(self as *const XmlToken as *const u8)
        }
    }

    pub(crate) fn as_literal(&self) -> String {
        match self {
            XmlToken::Literal(s) => s.clone(),
            _ => "".to_string()
        }
    }

    pub(crate) fn as_code(&self) -> String {
        match self {
            XmlToken::Code(s) => s.clone(),
            _ => "".to_string()
        }
    }

    pub(crate) fn is(&self, ty: XmlTokenType) -> bool {
        self.ordinal() == ty.ordinal()
    }
}

#[repr(u8)]
#[derive(Debug, Clone)]
enum XmlTokenType {
    LeftAngleBracket,
    RightAngleBracket,
    Literal,
    Code,
    Equals,
    LeftBrace,
    RightBrace,
    Slash,
    Quote
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
    in_literal: bool,
    putback: Vec<XmlToken>
}

impl XmlLexer {
    pub fn new(input: String) -> Self {
        Self {
            input,
            idx: 0,
            in_code_block: false,
            in_literal: false,
            putback: vec![],
        }
    }

    fn next_char(&mut self) -> Result<char, String> {
        self.idx += 1;
        self.input.chars().nth(self.idx - 1).ok_or(format!("Unexpected end at position {}", self.idx + 1).to_string())
    }

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

        if self.in_literal {
            let mut str = String::new();
            loop {
                let n = self.next_char()?;
                if n == '\\' {
                    let n = self.next_char()?;
                    str.push(n);
                } else {
                    if n == '"' {
                        self.putback(XmlToken::Quote);
                        self.in_literal = false;
                        return Ok(XmlToken::Literal(str));
                    }
                    str.push(n);
                }
            }
        }

        if self.in_code_block {
            let mut code = String::new();
            let mut brace_count = 1;
            loop {
                let n = self.next_char()?;
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

        let mut next = self.next_char()?;
        if next.is_whitespace() {
            next = self.next_char()?;
        }

        match next {
            '<' => Ok(XmlToken::LeftAngleBracket),
            '>' => Ok(XmlToken::RightAngleBracket),
            '/' => Ok(XmlToken::Slash),
            '"' => {
                self.in_literal = true;
                Ok(XmlToken::Quote)
            }
            '{' => {
                self.in_code_block = true;
                Ok(XmlToken::LeftBrace)
            }
            '}' => Ok(XmlToken::RightBrace),
            '=' => Ok(XmlToken::Equals),
            _ => {
                let mut str = String::new();
                str.push(next);
                loop {
                    let n = self.next_char()?;
                    if !n.is_alphanumeric() && n != '_' {
                        self.idx -= 1;
                        return Ok(XmlToken::Literal(str));
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