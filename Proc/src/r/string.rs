use ui_parsing::xml::{Entity, XmlValue};

pub enum ParsedString {
    S(String),
    File(String),
}

pub fn parse_string(entity: &Entity) -> (String, ParsedString) {
    if entity.name().as_str() != "string" {
        panic!(
            "String resource must be named string, got {}!",
            entity.name()
        );
    }
    if let Some(XmlValue::Str(name)) = entity.get_attrib("name") {
        if let Some(XmlValue::Str(val)) = entity.get_attrib("val") {
            return (name.clone(), ParsedString::S(val.clone()));
        } else if let Some(XmlValue::Str(file)) = entity.get_attrib("file") {
            return (name.clone(), ParsedString::File(file.clone()));
        }
    }
    panic!("Illegal format for string resource! Expected <string name=\"\" val=\"\">");
}
