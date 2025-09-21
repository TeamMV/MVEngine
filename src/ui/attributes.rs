use crate::ui::utils::ToRope;
use mvutils::state::{MappedState, State};
use ropey::Rope;
use std::str::FromStr;

pub type UiState = MappedState<Rope, Rope>;

#[derive(Clone)]
pub struct Attributes {
    pub elem_type: String,
    pub classes: Vec<String>,
    pub id: Option<String>,
}

impl Attributes {
    pub fn new(elem_type: &str) -> Self {
        Self {
            elem_type: elem_type.to_string(),
            classes: vec![],
            id: None,
        }
    }

    pub fn with_id<T: IntoAttrib<String>>(&mut self, id: T) {
        self.id = Some(id.into_attrib());
    }

    pub fn with_class<T: IntoAttrib<String>>(&mut self, class: T) {
        self.classes.push(class.into_attrib());
    }

    pub fn with_classes(&mut self, classes: &[String]) {
        self.classes.extend_from_slice(classes);
    }
}

pub trait IntoAttrib<T> {
    fn into_attrib(self) -> T;
}

impl<T> IntoAttrib<T> for T {
    fn into_attrib(self) -> T {
        self
    }
}

impl<T> IntoAttrib<State<T>> for T {
    fn into_attrib(self) -> State<T> {
        State::new(self)
    }
}

impl<T: Clone> IntoAttrib<MappedState<T, T>> for T {
    fn into_attrib(self) -> MappedState<T, T> {
        State::new(self).map_identity()
    }
}

impl IntoAttrib<String> for &str {
    fn into_attrib(self) -> String {
        self.to_string()
    }
}

impl IntoAttrib<State<String>> for &str {
    fn into_attrib(self) -> State<String> {
        State::new(self.to_string())
    }
}

impl IntoAttrib<State<Rope>> for &str {
    fn into_attrib(self) -> State<Rope> {
        State::new(Rope::from_str(self))
    }
}

impl IntoAttrib<UiState> for &str {
    fn into_attrib(self) -> UiState {
        State::new(Rope::from_str(self)).map_identity()
    }
}

impl IntoAttrib<UiState> for State<Rope> {
    fn into_attrib(self) -> UiState {
        self.map_identity()
    }
}

impl IntoAttrib<UiState> for String {
    fn into_attrib(self) -> UiState {
        State::new(self.to_rope()).map_identity()
    }
}