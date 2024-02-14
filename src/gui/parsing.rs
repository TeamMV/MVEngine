use crate::gui::elements::GuiElementCallbacks;

pub struct Unparsed {
    name: String,
    attributes: hashbrown::HashMap<String, String>,
    children: Vec<Unparsed>,
}

pub trait GuiParsable<T: GuiElementCallbacks> {
    fn try_parse(&self, unparsed: &Unparsed) -> Option<T>;
}

pub(crate) struct GuiParseSpec {
    tag_name: String,
    attributes: Vec<String>
}

pub(crate) struct GuiElementParser {
    parser: u32
}