use hashbrown::HashMap;
use mvutils::state::{MappedState, State};

pub type UiState = MappedState<String, String>;

#[derive(Clone)]
pub struct Attributes {
    pub elem_type: String,
    pub classes: Vec<String>,
    pub id: Option<String>,
    pub attribs: HashMap<String, AttributeValue>,
    pub inner: Option<AttributeValue>,
    //pub children: Option<Vec<VNode>>,
}

impl Attributes {
    pub fn new(elem_type: &str) -> Self {
        Self {
            elem_type: elem_type.to_string(),
            classes: vec![],
            id: None,
            attribs: HashMap::new(),
            inner: None,
            //children: None,
        }
    }

    pub fn with_id(&mut self, id: String) {
        self.id = Some(id);
    }

    pub fn with_class(&mut self, class: String) {
        self.classes.push(class);
    }

    pub fn with_classes(&mut self, classes: &[String]) {
        self.classes.extend_from_slice(classes);
    }

    pub fn with_attrib(&mut self, name: String, value: AttributeValue) {
        if let AttributeValue::Str(ref s) = value {
            if name == "id".to_string() {
                self.id = Some(s.clone());
                return;
            }
            if name == "class".to_string() {
                self.classes
                    .extend(s.split_whitespace().map(|st| st.to_string()));
                return;
            }
        }

        self.attribs.insert(name, value);
    }
    pub fn with_inner(&mut self, value: AttributeValue) {
        self.inner = Some(value);
    }
}

#[derive(Clone)]
pub enum AttributeValue {
    Str(String),
    State(MappedState<String, String>),
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
}

impl AttributeValue {
    pub fn as_string(&self) -> String {
        match self {
            AttributeValue::Str(s) => s.clone(),
            AttributeValue::State(s) => s.read().clone(),
            AttributeValue::Int(i) => i.to_string(),
            AttributeValue::Float(f) => f.to_string(),
            AttributeValue::Bool(b) => b.to_string(),
            AttributeValue::Char(c) => c.to_string(),
        }
    }

    pub fn as_ui_state(&self) -> UiState {
        match self {
            AttributeValue::Str(s) => State::new(s.clone()).map_identity(),
            AttributeValue::Int(i) => State::new(i.to_string()).map_identity(),
            AttributeValue::Float(f) => State::new(f.to_string()).map_identity(),
            AttributeValue::Bool(b) => State::new(b.to_string()).map_identity(),
            AttributeValue::Char(c) => State::new(c.to_string()).map_identity(),
            AttributeValue::State(state) => state.clone(),
        }
    }
}

pub trait ToAttrib {
    fn to_attrib(self) -> AttributeValue;
}

impl ToAttrib for String {
    fn to_attrib(self) -> AttributeValue {
        AttributeValue::Str(self)
    }
}

impl ToAttrib for &str {
    fn to_attrib(self) -> AttributeValue {
        AttributeValue::Str(self.to_string())
    }
}

impl ToAttrib for State<String> {
    fn to_attrib(self) -> AttributeValue {
        let state = self.map(|s| s.to_string());
        AttributeValue::State(state)
    }
}

impl ToAttrib for MappedState<String, String> {
    fn to_attrib(self) -> AttributeValue {
        AttributeValue::State(self)
    }
}

macro_rules! to_attrib_int {
    ($($t:ty,)*) => {
        $(
            impl ToAttrib for $t {
                fn to_attrib(self) -> AttributeValue {
                    AttributeValue::Int(self as i64)
                }
            }
        )*
    };
}

to_attrib_int!(
    u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize,
);

macro_rules! to_attrib_float {
    ($($t:ty,)*) => {
        $(
            impl ToAttrib for $t {
                fn to_attrib(self) -> AttributeValue {
                    AttributeValue::Float(self as f64)
                }
            }
        )*
    };
}

to_attrib_float!(f32, f64,);

impl ToAttrib for bool {
    fn to_attrib(self) -> AttributeValue {
        AttributeValue::Bool(self)
    }
}

impl ToAttrib for char {
    fn to_attrib(self) -> AttributeValue {
        AttributeValue::Char(self)
    }
}
