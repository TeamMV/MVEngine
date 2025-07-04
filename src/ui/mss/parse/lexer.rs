use std::collections::VecDeque;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Clone, Debug)]
pub enum MSSToken {
    EOF,
    Error(String),

    Star,
    Type,
    Class,
    Id,
    Comma,

    StyleBlock(String),

    Ident(String),
}

pub struct MSSLexer<'a> {
    chars: Peekable<Chars<'a>>,
    putback: VecDeque<MSSToken>,
}

impl<'a> MSSLexer<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            chars: s.chars().peekable(),
            putback: VecDeque::new(),
        }
    }

    fn collect_until<P: Fn(char) -> bool>(&mut self, predicate: P) -> String {
        let mut s = String::new();
        let mut next = self.chars.peek().cloned();
        while let Some(n) = next {
            if predicate(n) {
                return s;
            }
            self.chars.next();
            s.push(n);
            next = self.chars.peek().cloned();
        }
        s
    }

    pub fn next(&mut self) -> MSSToken {
        if !self.putback.is_empty() {
            return self.putback.pop_front().unwrap();
        }

        //skip whitespace
        let _ = self.collect_until(|x| !x.is_whitespace());
        let next = self.chars.next();

        if let Some(n) = next {
            return match n {
                '*' => MSSToken::Star,
                ',' => MSSToken::Comma,
                '{' => {
                    let block = self.collect_until(|x| x == '}');
                    self.chars.next(); //Consume }
                    MSSToken::StyleBlock(block)
                }
                _ => {
                    //word
                    let mut s = String::new();
                    s.push(n);
                    s += &*self.collect_until(|x| !(x.is_alphanumeric() || x == '_'));
                    match s.as_str() {
                        "type" => MSSToken::Type,
                        "class" => MSSToken::Class,
                        "id" => MSSToken::Id,
                        _ => MSSToken::Ident(s),
                    }
                }
            };
        }

        MSSToken::EOF
    }
}
