use ui_parsing::xml::{Entity, XmlValue};

pub fn parse_color(entity: &Entity) -> (String, String) {
    if entity.name().as_str() != "color" {
        panic!("Color resource must be named color, got {}!", entity.name());
    }
    if let Some(val) = entity.get_attrib("val") {
        if let Some(name) = entity.get_attrib("name") {
            if let XmlValue::Str(val_s) = val {
                if let XmlValue::Str(name_s) = name {
                    return (name_s.clone(), val_s.clone());
                }
            }
            panic!("Code blocks are not supported in color!")
        } else {
            panic!("Expected a 'name' attribute on color!")
        }
    } else {
        panic!("Expected a 'val' attribute on color!")
    }
}