use std::{mem, ptr};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use mvutils::utils::Recover;
use crate::err::panic;

pub trait Parser {
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
        if let Some(attrib) = attrib {
            T::from_str(attrib).ok()
        } else {
            None
        }
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