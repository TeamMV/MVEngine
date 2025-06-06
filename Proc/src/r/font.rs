use ui_parsing::xml::{Entity, XmlValue};

pub fn parse_font(entity: &Entity) -> (String, String){
    if entity.name().as_str() != "font" {
        panic!("Font resource must be named font, got {}!", entity.name());
    }
    if let Some(val) = entity.get_attrib("src") {
        if let Some(name) = entity.get_attrib("name") {
            if let XmlValue::Str(val_s) = val {
                if let XmlValue::Str(name_s) = name {
                    if let Some(XmlValue::Str(atlas)) = entity.get_attrib("atlas") {
                        let lit = format!("{val_s}|{atlas}");
                        return (name_s.clone(), lit);
                    }
                }
            }
            panic!("Code blocks are not supported in font!")
        } else {
            panic!("Expected a 'name' attribute on font!")
        }
    } else {
        panic!("Expected a 'src' attribute on font!")
    }
}