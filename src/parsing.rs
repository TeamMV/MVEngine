use std::str::FromStr;
use roxmltree::{Children, Document, Node};
use crate::err::panic;

pub trait Parser<'a> {
    fn parse(raw: Vec<u8>) -> Result<Self, String>
    where Self: Sized;

    fn advance(&mut self) -> bool;

    fn root(&self) -> &str;

    fn current(&self) -> Option<&str>;

    fn text(&self) -> Option<&str>;

    fn attrib(&self, name: String) -> Option<&str>;

    fn t_attrib<T>(&self, name: String) -> Option<T>
    where T: FromStr, <T as FromStr>::Err: std::fmt::Debug {
        let attrib = self.attrib(name);
        if attrib.is_some() {
            let r = T::from_str(attrib.unwrap());
            if r.is_ok() { return Some(r.unwrap()); }
            return None;
        }
        None
    }

    fn has_attrib(&self, name: String) -> bool {
        self.attrib(name).is_some()
    }

    fn has_inner(&self) -> bool;

    fn inner(&self) -> Option<Self>
    where Self: Sized;

    fn require_root(self, root: String) -> Option<Self>
    where Self: Sized {
        let self_root = self.root();
        if !root.eq(&self_root) {
            panic!("Expected root {root}, found {self_root}");
        }
        Some(self)
    }
}

pub struct XMLParser<'a> {
    raw: String,
    root: String,
    root_elem: Node<'a, 'a>,
    current_elem: Option<Node<'a, 'a>>,
    elem_list: Children<'a, 'a>,
}

impl<'a> Parser<'a> for XMLParser<'a> {
    fn parse(raw: Vec<u8>) -> Result<Self, String> {
        let str = String::from_utf8(raw);
        if str.is_err() {
            return Err("Could not read bytes into String!".to_string());
        }
        let str = str.unwrap();
        let doc = Document::parse(str.as_str());
        if doc.is_err() {
            return Err("Could not parse XML!".to_string());
        }

        let doc = doc.unwrap();
        let root = doc.root().tag_name().name().to_string();
        let root_elem = doc.root_element();
        let children = root_elem.children();
        let current_elem = None;

        let inst = Self {
            raw: str,
            root,
            root_elem,
            current_elem,
            elem_list: children,
        };

        Ok(inst)
    }

    fn advance(&mut self) -> bool {
        let next = self.elem_list.next();
        if next.is_some() {
            self.current_elem = next;
            return true;
        }
        false
    }

    fn root(&self) -> &str {
        self.root.as_str()
    }

    fn current(&self) -> Option<&str> {
        if self.current_elem.is_some() {
            return Some(self.current_elem.unwrap().tag_name().name());
        }
        None
    }

    fn text(&self) -> Option<&str> {
        if self.current_elem.is_some() {
            return self.current_elem.unwrap().text();
        }
        None
    }

    fn attrib(&self, name: String) -> Option<&str> {
        if self.current_elem.is_some() {
            return self.current_elem.unwrap().attribute(name.as_str());
        }
        None
    }

    fn has_inner(&self) -> bool {
        if self.current_elem.is_some() {
            return self.current_elem.unwrap().has_children();
        }
        false
    }

    fn inner(&self) -> Option<Self> {
        if self.current_elem.is_some() {
            let children = self.current_elem.unwrap().children();
            let inst = Self {
                raw: self.raw.clone(),
                root: self.root.clone(),
                root_elem: self.root_elem,
                current_elem: None,
                elem_list: children,
            };
            return Some(inst);
        }
        None
    }
}