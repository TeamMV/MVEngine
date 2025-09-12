use hashbrown::HashMap;
use mvutils::state::{MappedState, State};
use ropey::{Rope, RopeSlice};
use crate::ui::parse::parse_num;

pub type UiState = MappedState<Rope, Rope>;

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
                self.id = Some(s.to_string());
                return;
            }
            if name == "class".to_string() {
                self.classes
                    .extend(s.to_string().split_whitespace().map(|st| st.to_string()));
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
    Str(Rope),
    State(UiState),
    BoolState(State<bool>),
    FloatState(State<f32>),
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
}

impl AttributeValue {
    pub fn as_rope(&self) -> Rope {
        match self {
            AttributeValue::Str(s) => s.clone(),
            AttributeValue::State(s) => s.read().clone(),
            AttributeValue::Int(i) => i.to_rope(),
            AttributeValue::Float(f) => f.to_rope(),
            AttributeValue::Bool(b) => b.to_rope(),
            AttributeValue::Char(c) => c.to_rope(),
            AttributeValue::BoolState(b) => b.read().to_rope(),
            AttributeValue::FloatState(f) => f.read().to_rope()
        }
    }

    pub fn as_ui_state(&self) -> UiState {
        match self {
            AttributeValue::Str(s) => State::new(s.to_rope()).map_identity(),
            AttributeValue::Int(i) => State::new(i.to_rope()).map_identity(),
            AttributeValue::Float(f) => State::new(f.to_rope()).map_identity(),
            AttributeValue::Bool(b) => State::new(b.to_rope()).map_identity(),
            AttributeValue::Char(c) => State::new(c.to_rope()).map_identity(),
            AttributeValue::State(state) => state.clone(),
            AttributeValue::BoolState(b) => State::new(b.read().to_rope()).map_identity(),
            AttributeValue::FloatState(f) => State::new(f.read().to_rope()).map_identity()
        }
    }

    pub fn as_bool_state(&self) -> State<bool> {
        match self {
            AttributeValue::Str(s) => State::new(false),
            AttributeValue::Int(i) => State::new(false),
            AttributeValue::Float(f) => State::new(false),
            AttributeValue::Bool(b) => State::new(*b),
            AttributeValue::Char(c) => State::new(*c == 'y'),
            AttributeValue::State(_) => State::new(false),
            AttributeValue::BoolState(b) => b.clone(),
            &AttributeValue::FloatState(_) => State::new(false)
        }
    }

    pub fn as_float_state(&self) -> State<f32> {
        match self {
            AttributeValue::Str(s) => State::new(parse_num(&s.to_string()).unwrap_or_default()),
            AttributeValue::Int(i) => State::new(*i as f32),
            AttributeValue::Float(f) => State::new(*f as f32),
            AttributeValue::Bool(_) => State::new(0.0),
            AttributeValue::Char(_) => State::new(0.0),
            AttributeValue::State(state) => State::new(parse_num(&state.read().to_string()).unwrap_or_default()),
            AttributeValue::BoolState(_) => State::new(0.0),
            AttributeValue::FloatState(f) => f.clone()
        }
    }
}

pub trait ToAttrib {
    fn to_attrib(self) -> AttributeValue;
}

impl ToAttrib for String {
    fn to_attrib(self) -> AttributeValue {
        AttributeValue::Str(self.to_rope())
    }
}

impl ToAttrib for &str {
    fn to_attrib(self) -> AttributeValue {
        AttributeValue::Str(self.to_rope())
    }
}

impl ToAttrib for Rope {
    fn to_attrib(self) -> AttributeValue {
        AttributeValue::Str(self)
    }
}

impl ToAttrib for State<Rope> {
    fn to_attrib(self) -> AttributeValue {
        let state = self.map_identity();
        AttributeValue::State(state)
    }
}

impl ToAttrib for State<bool> {
    fn to_attrib(self) -> AttributeValue {
        AttributeValue::BoolState(self)
    }
}

impl ToAttrib for State<f32> {
    fn to_attrib(self) -> AttributeValue {
        AttributeValue::FloatState(self)
    }
}

impl ToAttrib for UiState {
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

pub trait ToRope {
    fn to_rope(&self) -> Rope;
}

impl<T: ToString> ToRope for T {
    fn to_rope(&self) -> Rope {
        let s = self.to_string();
        Rope::from(s)
    }
}