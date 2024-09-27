pub trait Component {
    fn get_base(& self) -> & ComponentBase;

    fn get_base_mut(&mut self) -> &mut ComponentBase;
}

pub struct ComponentBase {
    name: String,
}

impl ComponentBase {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}