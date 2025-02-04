use hashbrown::HashMap;
use crate::ui::rendering::shapes::lexer::{NumberLit, Token, TokenStream};

pub mod lexer;
pub mod shape_gen;
pub mod polygon;
pub mod modifier;
mod geometry;

type Ast = Vec<Command>;

#[derive(Debug)]
pub enum Command {
    Assign(String, Assignment),
    Call(String, Vec<Param>),
    Select(String)
}

#[derive(Debug)]
enum Assignment {
    New(ParsedStruct),
    Clone(String)
}

#[derive(Debug, Clone)]
pub(crate) enum Param {
    Str(String),
    Struct(ParsedStruct)
}

impl Param {
    pub fn as_str(&self) -> &String {
        match self {
            Param::Str(s) => s,
            Param::Struct(_) => unreachable!()
        }
    }

    pub fn as_struct(&self) -> &ParsedStruct {
        match self {
            Param::Str(_) => unreachable!(),
            Param::Struct(s) => s
        }
    }
}

pub struct ShapeParser;

impl ShapeParser {
    pub fn parse(shape_expr: &str) -> Result<Ast, String> {
        let mut tokens = TokenStream::tokenize(shape_expr);

        let mut ast = Ast::new();

        while let Some(token) = tokens.next() {
            match token {
                Token::Identifier(ident) => {
                    let ident_next = tokens.next();
                    if let Some(next_token) = ident_next {
                        if let Token::Equals = next_token {
                            let name_token = tokens.expect_next_ident()?;
                            let decider_token = tokens.next();
                            if let Some(decider_token) = decider_token {
                                if let Token::LBracket = decider_token {
                                    //struct
                                    tokens.putback(Token::Identifier(name_token));
                                    tokens.putback(Token::LBracket);
                                    let parsed = Self::parse_struct(&mut tokens)?;
                                    tokens.expect_next_token(Token::Semicolon)?;
                                    ast.push(Command::Assign(ident, Assignment::New(parsed)));
                                } else {
                                    //other
                                    tokens.putback(decider_token);
                                    tokens.expect_next_token(Token::Semicolon)?;
                                    ast.push(Command::Assign(ident, Assignment::Clone(name_token)));
                                }
                            } else {
                                ast.push(Command::Assign(ident, Assignment::Clone(name_token)));
                            }
                        } else if let Token::Identifier(name) = next_token {
                            tokens.putback(Token::Identifier(name));
                            let mut params = vec![];
                            while let Some(token) = tokens.next() {
                                if let Token::Semicolon = token {
                                    ast.push(Command::Call(ident, params));
                                    break;
                                } else {
                                    tokens.putback(token);
                                }
                                let param_ident = tokens.expect_next_ident()?;
                                let next = tokens.expect_next_some()?;
                                if let Token::LBracket = next {
                                    //struct
                                    tokens.putback(Token::Identifier(param_ident));
                                    tokens.putback(Token::LBracket);
                                    let parsed = Self::parse_struct(&mut tokens)?;;
                                    params.push(Param::Struct(parsed));
                                } else {
                                    tokens.putback(next);
                                    params.push(Param::Str(param_ident));
                                }
                            }
                        } else if let Token::Semicolon = next_token {
                            ast.push(Command::Call(ident, Vec::new()));
                        } else {
                            return Err(format!("Unexpected Token4: {:?}", next_token));
                        }
                    } else {
                        return Err("Expected Semicolon".to_string());
                    }
                }
                Token::Selector(ident) => {
                    tokens.expect_next_token(Token::Semicolon)?;
                    ast.push(Command::Select(ident));
                }
                Token::Error(e) => return Err(e),
                _ => return Err(format!("Unexpected Token3: {:?}", token))
            }
        }

        Ok(ast)
    }

    fn parse_struct(stream: &mut TokenStream) -> Result<ParsedStruct, String> {
        let struct_name = stream.expect_next_ident()?;
        let mut parsed_struct = ParsedStruct::new(struct_name);

        stream.expect_next_token(Token::LBracket)?;
        loop {
            let next = stream.expect_next_some()?;
            match next {
                Token::Identifier(name) => {
                    let value_next = stream.expect_next_some()?;
                    match value_next {
                        Token::Number(num) => {
                            parsed_struct.values.insert(name, StructValue::Number(num));
                        },
                        Token::LBracket => {
                            stream.putback(Token::Identifier(name.clone()));
                            stream.putback(Token::LBracket);
                            let inner_struct = Self::parse_struct(stream)?;
                            parsed_struct.values.insert(name, StructValue::Struct(Box::new(inner_struct)));
                        },
                        _ => return Err(format!("Unexpected Token2: {:?}", value_next))
                    }
                },
                Token::RBracket => return Ok(parsed_struct),
                Token::Error(e) => return Err(e),
                _ => return Err(format!("Unexpected Token1: {:?}", next))
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ParsedStruct {
    name: String,
    values: HashMap<String, StructValue>
}

impl ParsedStruct {
    fn new(name: String) -> Self {
        Self {
            name,
            values: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum StructValue {
    Number(NumberLit),
    Struct(Box<ParsedStruct>)
}