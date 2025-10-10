use crate::r::shape::{ParsedShape, ShapeLan};
use std::collections::HashMap;
use std::str::FromStr;
use ui_parsing::xml::{Entity, XmlValue};

pub fn parse_adaptive(entity: &Entity) -> (String, ParsedShape) {
    if entity.name().as_str() != "adaptive" {
        panic!(
            "Adaptive resource must be named string, got {}!",
            entity.name()
        );
    }
    if let Some(val) = entity.get_attrib("src") {
        if let Some(name) = entity.get_attrib("name") {
            if let XmlValue::Str(val_s) = val {
                if let XmlValue::Str(name_s) = name {
                    if let XmlValue::Str(lan) = entity
                        .get_attrib("language")
                        .unwrap_or(&XmlValue::Str("MSF".to_string()))
                    {
                        if let Ok(lan) = ShapeLan::from_str(lan) {
                            let mut inputs = HashMap::new();
                            if let Some(XmlValue::Entities(inner)) = entity.inner() {
                                for prop in inner {
                                    let name = prop.name().clone();
                                    if let Some(XmlValue::Str(s)) = prop.get_attrib("val") {
                                        inputs.insert(name, s.clone());
                                    }
                                }
                            }

                            return (
                                name_s.clone(),
                                ParsedShape {
                                    file: val_s.clone(),
                                    language: lan,
                                    inputs,
                                },
                            );
                        } else {
                            panic!("Illegal shape language: {lan}. Choose either MSFX or MSFX")
                        }
                    } else {
                        panic!("Adaptive requires a language attribute!")
                    }
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
