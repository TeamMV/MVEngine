use crate::math::vec::Vec2;
use hashbrown::HashMap;
use hashbrown::hash_map::Values;
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct ParsedRig {
    pub parts: Parsed<ParsedPart>,
    pub bones: Parsed<ParsedBone>,
    pub joints: Parsed<ParsedJoint>,
}

#[derive(Clone, Debug)]
pub struct ParsedPart {
    pub name: String,
    pub size: Vec2,
    pub anchor: Vec2,
    pub bone: Option<String>,
}

#[derive(Clone, Debug)]
pub enum BoneStart {
    Other(String),
    Point(Vec2),
}

#[derive(Clone, Debug)]
pub struct ParsedBone {
    pub name: String,
    pub start: BoneStart,
    pub end: Vec2,
}

#[derive(Clone, Debug)]
pub struct ParsedJoint {
    pub name: String,
    pub bone1: String,
    pub bone2: String,
}

#[derive(Clone, Debug)]
pub struct Parsed<T: Debug> {
    pub map: HashMap<String, T>,
}

impl<T: Debug> Parsed<T> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn find(&self, name: &str) -> Result<&T, String> {
        self.map.get(name).ok_or(format!("Unknown: {name}"))
    }

    pub fn find_mut(&mut self, name: &str) -> Result<&mut T, String> {
        self.map.get_mut(name).ok_or(format!("Unknown: {name}"))
    }

    pub fn verify(&self, name: &str) -> Result<(), String> {
        if !self.map.contains_key(name) {
            return Err(format!("Unknown: {name}"));
        }
        Ok(())
    }
}
