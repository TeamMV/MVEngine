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
                self.classes.extend(s.split_whitespace().map(|st| st.to_string()));
                return;
            }
            return;
        }

        self.attribs.insert(name, value);
    }
    pub fn with_inner(&mut self, value: AttributeValue) {
        self.inner = Some(value);
    }
}

pub enum AttributeValue {
    Str(String),
    Code(Box<dyn FnMut(&mut UiElement)>)
}