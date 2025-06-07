use ui_parsing::xml::{Entity, XmlValue};

pub enum GeomType {
    Shape,
    Adaptive,
}

pub struct ParsedGeometry {
    pub geom_type: GeomType,
    pub thingies: Vec<String>
}

pub fn parse_geometry(entity: &Entity) -> (String, ParsedGeometry) {
    if entity.name().as_str() != "geometry" {
        panic!("Geometry resource must be named geometry, got {}!", entity.name());
    }
    if let Some(XmlValue::Str(name)) = entity.get_attrib("name") {
        if let Some(XmlValue::Str(t)) = entity.get_attrib("type") {
            match t.as_str() {
                "shape" => {
                    let Some(XmlValue::Str(val)) = entity.get_attrib("ref") else { panic!("shape geometry needs a shape ref") };
                    return (name.clone(), ParsedGeometry { geom_type: GeomType::Shape, thingies: vec![val.clone()] });
                },
                "adaptive" => {
                    let Some(XmlValue::Str(val)) = entity.get_attrib("ref") else { panic!("adaptive geometry needs a adaptive ref") };
                    return (name.clone(), ParsedGeometry { geom_type: GeomType::Adaptive, thingies: vec![val.clone()] });
                },
                _ => panic!("Illegal geometry type {t}!")
            }
        }
    }

    panic!("Illegal Geometry setup! Expected name, type!")
}