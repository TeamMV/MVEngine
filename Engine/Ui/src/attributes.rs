pub struct Attributes {
    pub classes: Vec<String>,
    pub id: Option<String>,
}

impl Attributes {
    pub fn new() -> Self {
        Self {
            classes: vec![],
            id: None,
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
}
