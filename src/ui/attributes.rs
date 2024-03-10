pub struct Attributes {
    classes: Vec<String>,
    id: Option<String>,
}

impl Attributes {
    pub fn new() -> Self {
        Self {
            classes: vec![],
            id: None,
        }
    }
}
