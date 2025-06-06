use ui_parsing::xml::{Entity, XmlValue};

pub fn parse_shape(entity: &Entity) -> (String, String){
    if entity.name().as_str() != "shape" {
        panic!("Shape resource must be named shape, got {}!", entity.name());
    }
    if let Some(val) = entity.get_attrib("src") {
        if let Some(name) = entity.get_attrib("name") {
            if let XmlValue::Str(val_s) = val {
                if let XmlValue::Str(name_s) = name {
                    return (name_s.clone(), val_s.clone());
                }
            }
            panic!("Code blocks are not supported in shape!")
        } else {
            panic!("Expected a 'name' attribute on shape!")
        }
    } else {
        panic!("Expected a 'src' attribute on shape!")
    }
}