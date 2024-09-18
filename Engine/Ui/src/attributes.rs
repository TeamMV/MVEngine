use hashbrown::HashMap;
use crate::elements::UiElement;

pub struct Attributes {
    pub classes: Vec<String>,
    pub id: Option<String>,
    pub attribs: HashMap<String, AttributeValue>,
    pub inner: Option<AttributeValue>
}

impl Attributes {
    pub fn new() -> Self {
        Self {
            classes: vec![],
            id: None,
            attribs: HashMap::new(),
            inner: None,
        }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_class(mut self, class: String) -> Self {
        self.classes.push(class);
        self
    }

    pub fn with_classes(mut self, classes: &[String]) -> Self {
        self.classes.extend_from_slice(classes);
        self
    }

    pub fn with_attrib(mut self, name: String, value: AttributeValue) -> Self {
        if let AttributeValue::Str(ref s) = value {
            if name == "id".to_string() {
                self.id = Some(s.clone());
                return self;
            }
            if name == "class".to_string() {
                self.classes.extend(s.split_whitespace().map(|st| st.to_string()));
                return self;
            }
            return self;
        }

        self.attribs.insert(name, value);
        self
    }
    pub fn with_inner(mut self, value: AttributeValue) -> Self {
        self.inner = Some(value);
        self
    }
}

pub enum AttributeValue {
    Str(String),
    Code(Box<dyn FnMut(&mut UiElement)>)
}