use std::str::FromStr;
use mvutils::TryFromString;
use ui_parsing::xml::{Entity, XmlValue};

#[derive(TryFromString)]
pub enum ShapeLan {
    MSF,
    MSFX
}

pub struct ParsedShape {
    pub file: String,
    pub language: ShapeLan
}

pub fn parse_shape(entity: &Entity) -> (String, ParsedShape){
    if entity.name().as_str() != "shape" {
        panic!("Shape resource must be named shape, got {}!", entity.name());
    }
    if let Some(val) = entity.get_attrib("src") {
        if let Some(name) = entity.get_attrib("name") {
            if let XmlValue::Str(val_s) = val {
                if let XmlValue::Str(name_s) = name {
                    if let Some(XmlValue::Str(lan)) = entity.get_attrib("language") {
                        if let Ok(lan) = ShapeLan::from_str(lan) {
                            return (name_s.clone(), ParsedShape {
                                file: val_s.clone(),
                                language: lan,
                            });
                        } else {
                            panic!("Illegal shape language: {lan}. Choose either msf or msfx")
                        }
                    } else {
                        panic!("Shape requires a language attribute!")
                    }
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