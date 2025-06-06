use ui_parsing::xml::{Entity, XmlValue};

pub fn parse_adaptive(entity: &Entity) -> (String, String){
    if entity.name().as_str() != "adaptive" {
        panic!("Adaptive resource must be named adaptive, got {}!", entity.name());
    }
    if let Some(val) = entity.get_attrib("src") {
        if let Some(name) = entity.get_attrib("name") {
            if let XmlValue::Str(val_s) = val {
                if let XmlValue::Str(name_s) = name {
                    return (name_s.clone(), val_s.clone());
                }
            }
            panic!("Code blocks are not supported in adaptive!")
        } else {
            panic!("Expected a 'name' attribute on adaptive!")
        }
    } else {
        panic!("Expected a 'src' attribute on adaptive!")
    }
}