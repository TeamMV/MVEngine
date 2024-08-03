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
}
