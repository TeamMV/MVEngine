use std::collections::{HashMap};
use std::fmt::{Debug};
use std::marker::PhantomData;

pub struct XMLParser<S: ParserSecurity> {
    to_parse: Option<String>,
    result: Option<Tag>,
    phantom: PhantomData<S>,
}

pub struct Secure;

pub struct Unsecure;

pub trait ParserSecurity {}

impl ParserSecurity for Secure {}

impl ParserSecurity for Unsecure {}

impl<S: ParserSecurity> XMLParser<S> {
    pub fn new() -> Self {
        XMLParser::<S> { to_parse: None, result: None, phantom: Default::default() }
    }

    fn parse_tag(&self, xml: String) -> Result<Tag, String> {
        let mut tag = Tag {
            children: Vec::new(),
            name: String::new(),
            namespace: None,
            attributes: HashMap::new(),
        };

        let mut name_buf: String = String::new();
        let mut attrib_name_buf: String = String::new();
        let mut attrib_val_buf: String = String::new();
        let mut is_parse_val = false;
        let mut in_str = false;
        let mut name_comp = false;
        let mut top_comp = false;
        let mut attrib_comp = false;
        for c in xml.chars() {
            if c == '<' {
                continue;
            }
            if !top_comp && !name_comp && c.is_whitespace() {
                name_comp = true;
                self.set_name(&mut tag, name_buf.clone());
            }
            if c == '>' {
                top_comp = true;
                name_comp = true;
                self.set_name(&mut tag, name_buf.clone());
                if !attrib_name_buf.is_empty() {
                    attrib_comp = false;
                    self.add_attrib(&mut tag, attrib_name_buf.clone(), attrib_val_buf.clone());
                    attrib_name_buf.clear();
                    attrib_val_buf.clear();
                    is_parse_val = false;
                }
            } else if !name_comp {
                name_buf.push(c);
            } else if !top_comp {
                if c == '=' {
                    is_parse_val = true;
                    continue;
                }
                if !attrib_comp && !c.is_whitespace() || in_str {
                    if !is_parse_val {
                        attrib_name_buf.push(c);
                    } else {
                        if c == '\"' || c == '"' {
                            in_str = !in_str;
                            if !in_str {
                                attrib_comp = true;
                            }
                            continue;
                        }
                        attrib_val_buf.push(c);
                    }
                } else if attrib_comp {
                    attrib_comp = false;
                    self.add_attrib(&mut tag, attrib_name_buf.clone(), attrib_val_buf.clone());
                    attrib_name_buf.clear();
                    attrib_val_buf.clear();
                    is_parse_val = false;
                }
            }
        }

        return Ok(tag);
    }

    fn set_name(&self, tag: &mut Tag, s: String) {
        if !s.contains(":") {
            tag.name = s;
        } else {
            let mut r = s.split(":");
            tag.namespace = Some(r.next().unwrap_or("").to_string());
            tag.name = r.next().unwrap_or("").to_string();
        }
    }

    fn add_attrib(&self, tag: &mut Tag, name: String, value: String) {
        tag.attributes.insert(name, value);
    }
}

impl XMLParser<Secure> {
    pub fn parse_xml(&mut self, xml: &String) {
        self.to_parse = Some(xml.to_string());
        let res = self.parse_tag(xml.to_string());
        if res.is_ok() {
            self.result = Some(res.unwrap());
        }
    }
}

impl XMLParser<Unsecure> {
    pub fn parse_xml(&mut self, xml: &String) {
        self.to_parse = Some(xml.to_string());
        let res = self.parse_tag(xml.to_string());
        if res.is_ok() {
            self.result = Some(res.unwrap());
        } else {
            panic!("{}", res.err().unwrap());
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Tag<> {
    pub children: Vec<Tag>,
    pub name: String,
    pub namespace: Option<String>,
    pub attributes: HashMap<String, String>,
}

impl Tag {
    pub fn get_tag(&self, name: &str) -> Vec<&Tag> {
        let mut tags = Vec::new();
        for t in self.children.iter() {
            if t.name.as_str() == name {
                tags.push(t);
            }
        }
        tags
    }

    pub fn has_tag(&self, name: &str) -> bool {
        for t in self.children.iter() {
            if t.name.as_str() == name {
                return true;
            }
        }
        false
    }

    pub fn get_attrib(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }

    pub fn has_attrib(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    pub fn get_attrib_keys(&self) -> Vec<&String> {
        self.attributes.keys().collect()
    }

    pub fn get_children(&self) -> Vec<&Tag> {
        self.children.iter().collect()
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_namespace(&self) -> &Option<String> {
        &self.namespace
    }

    pub fn has_namespace(&self) -> bool {
        self.namespace.is_some()
    }
}
