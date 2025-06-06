use ui_parsing::xml::{Entity, XmlValue};

pub fn parse_texture(entity: &Entity) -> (String, String){
    if entity.name().as_str() != "texture" {
        panic!("Texture resource must be named texture, got {}!", entity.name());
    }
    if let Some(val) = entity.get_attrib("src") {
        if let Some(name) = entity.get_attrib("name") {
            if let XmlValue::Str(val_s) = val {
                if let XmlValue::Str(name_s) = name {
                    let mut sampler = format!("|nearest");
                    if let Some(XmlValue::Str(sam)) = entity.get_attrib("sampler") {
                        sampler = format!("|{sam}");
                    }
                    let mut cloned_val_s = val_s.clone();
                    cloned_val_s.push_str(sampler.as_str());
                    return (name_s.clone(), cloned_val_s);
                }
            }
            panic!("Code blocks are not supported in texture!")
        } else {
            panic!("Expected a 'name' attribute on texture!")
        }
    } else {
        panic!("Expected a 'src' attribute on texture!")
    }
}