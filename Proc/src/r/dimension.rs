use ui_parsing::xml::{Entity, XmlValue};

pub fn parse_dimension(entity: &Entity) -> (String, String) {
    if entity.name().as_str() != "dimension" {
        panic!(
            "Dimension resource must be named dimension, got {}!",
            entity.name()
        );
    }
    if let Some(XmlValue::Str(name)) = entity.get_attrib("name") {
        if let Some(XmlValue::Str(val)) = entity.get_attrib("val") {
            return (name.clone(), val.clone());
        }
    }
    panic!("Illegal format for dimension resource! Expected <dimension name=\"\" val=\"\">");
}
