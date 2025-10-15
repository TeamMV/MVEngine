use std::collections::HashMap;
use std::ops::Index;

#[derive(Debug, Clone)]
pub enum JsonElement {
    Str(String),
    Number(JsonNumber),
    Bool(bool),
    Object(JsonObject),
    Array(JsonArray),
}

impl JsonElement {
    pub fn as_string(&self) -> Option<String> {
        match self {
            JsonElement::Str(s) => Some(s.to_string()),
            JsonElement::Bool(b) => Some(b.to_string()),
            JsonElement::Number(JsonNumber::Int(i)) => Some(i.to_string()),
            JsonElement::Number(JsonNumber::Float(f)) => Some(f.to_string()),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<&JsonNumber> {
        match self {
            JsonElement::Number(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            JsonElement::Number(JsonNumber::Int(i)) => Some(*i),
            JsonElement::Number(JsonNumber::Float(f)) => Some(*f as i32),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            JsonElement::Number(JsonNumber::Float(f)) => Some(*f),
            JsonElement::Number(JsonNumber::Int(i)) => Some(*i as f32),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonElement::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&JsonObject> {
        match self {
            JsonElement::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&JsonArray> {
        match self {
            JsonElement::Array(a) => Some(a),
            _ => None,
        }
    }

    //lmao
    /// Convenient deep accessor: allows chained calls
    /// like `elem.convenient_deep_accessor("user.name.first")`
    pub fn convenient_deep_accessor(&self, path: &str) -> Option<&JsonElement> {
        let mut current = self;
        for part in path.split('.') {
            current = match current {
                JsonElement::Object(obj) => obj.get(part)?,
                JsonElement::Array(arr) => {
                    let idx = part.parse::<usize>().ok()?;
                    arr.get(idx)?
                }
                _ => return None,
            };
        }
        Some(current)
    }
}

#[derive(Debug, Clone)]
pub enum JsonNumber {
    Float(f32),
    Int(i32),
}

#[derive(Debug, Clone)]
pub struct JsonObject {
    map: HashMap<String, JsonElement>,
}

impl JsonObject {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub(super) fn field(&mut self, name: String, value: JsonElement) {
        self.map.insert(name, value);
    }

    pub fn get(&self, key: &str) -> Option<&JsonElement> {
        self.map.get(key)
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get(key)?.as_string()
    }

    pub fn get_int(&self, key: &str) -> Option<i32> {
        self.get(key)?.as_int()
    }

    pub fn get_float(&self, key: &str) -> Option<f32> {
        self.get(key)?.as_float()
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key)?.as_bool()
    }

    pub fn get_number(&self, key: &str) -> Option<&JsonNumber> {
        self.get(key)?.as_number()
    }

    pub fn get_object(&self, key: &str) -> Option<&JsonObject> {
        self.get(key)?.as_object()
    }

    pub fn get_array(&self, key: &str) -> Option<&JsonArray> {
        self.get(key)?.as_array()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.map.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &JsonElement> {
        self.map.values()
    }
}

impl Index<&str> for JsonObject {
    type Output = JsonElement;

    fn index(&self, key: &str) -> &Self::Output {
        &self.map[key]
    }
}

#[derive(Debug, Clone)]
pub struct JsonArray {
    vec: Vec<JsonElement>,
}

impl JsonArray {
    pub fn new() -> Self {
        Self { vec: vec![] }
    }

    pub(super) fn push(&mut self, value: JsonElement) {
        self.vec.push(value);
    }

    pub fn get(&self, index: usize) -> Option<&JsonElement> {
        self.vec.get(index)
    }

    pub fn get_string(&self, index: usize) -> Option<String> {
        self.get(index)?.as_string()
    }

    pub fn get_int(&self, index: usize) -> Option<i32> {
        self.get(index)?.as_int()
    }

    pub fn get_float(&self, index: usize) -> Option<f32> {
        self.get(index)?.as_float()
    }

    pub fn get_number(&self, index: usize) -> Option<&JsonNumber> {
        self.get(index)?.as_number()
    }

    pub fn get_object(&self, index: usize) -> Option<&JsonObject> {
        self.get(index)?.as_object()
    }

    pub fn get_array(&self, index: usize) -> Option<&JsonArray> {
        self.get(index)?.as_array()
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &JsonElement> {
        self.vec.iter()
    }
}

impl Index<usize> for JsonArray {
    type Output = JsonElement;

    fn index(&self, index: usize) -> &Self::Output {
        &self.vec[index]
    }
}

impl<'a> IntoIterator for &'a JsonArray {
    type Item = &'a JsonElement;
    type IntoIter = std::slice::Iter<'a, JsonElement>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.iter()
    }
}

impl IntoIterator for JsonArray {
    type Item = JsonElement;
    type IntoIter = std::vec::IntoIter<JsonElement>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}
