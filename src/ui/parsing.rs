use crate::ui::elements::UiElementCallbacks;

pub struct Unparsed {
    name: String,
    attributes: hashbrown::HashMap<String, String>,
    children: Vec<Unparsed>,
}

pub trait UiParsable<T: UiElementCallbacks> {
    fn try_parse(&self, unparsed: &Unparsed) -> Option<T>;
}

pub(crate) struct UiParseSpec {
    tag_name: String,
    attributes: Vec<String>,
}

pub(crate) struct UiElementParser {
    parser: u32,
}
