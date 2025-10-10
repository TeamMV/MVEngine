use ui_parsing::xml::{Entity, XmlValue};

pub enum DrawableType {
    Color,
    Texture,
    Animation,
    Tileset,
}

pub struct ParsedDrawable {
    pub drawable_type: DrawableType,
    pub thingies: Vec<String>,
}

pub fn parse_drawable(entity: &Entity) -> (String, ParsedDrawable) {
    if entity.name().as_str() != "drawable" {
        panic!(
            "Drawable resource must be named drawable, got {}!",
            entity.name()
        );
    }
    if let Some(XmlValue::Str(name)) = entity.get_attrib("name") {
        if let Some(XmlValue::Str(t)) = entity.get_attrib("type") {
            match t.as_str() {
                "color" => {
                    let Some(XmlValue::Str(val)) = entity.get_attrib("ref") else {
                        panic!("color drawable needs a color ref")
                    };
                    return (
                        name.clone(),
                        ParsedDrawable {
                            drawable_type: DrawableType::Color,
                            thingies: vec![val.clone()],
                        },
                    );
                }
                "texture" => {
                    let Some(XmlValue::Str(val)) = entity.get_attrib("ref") else {
                        panic!("texture drawable needs a texture ref")
                    };
                    return (
                        name.clone(),
                        ParsedDrawable {
                            drawable_type: DrawableType::Texture,
                            thingies: vec![val.clone()],
                        },
                    );
                }
                "animation" => {
                    let Some(XmlValue::Str(val)) = entity.get_attrib("ref") else {
                        panic!("animation drawable needs a animation ref")
                    };
                    return (
                        name.clone(),
                        ParsedDrawable {
                            drawable_type: DrawableType::Animation,
                            thingies: vec![val.clone()],
                        },
                    );
                }
                "tileset" => {
                    let Some(XmlValue::Str(val)) = entity.get_attrib("ref") else {
                        panic!("tileset drawable needs a tileset ref")
                    };

                    let Some(XmlValue::Str(tile)) = entity.get_attrib("tileref") else {
                        panic!("tileset drawable needs a tileref")
                    };
                    return (
                        name.clone(),
                        ParsedDrawable {
                            drawable_type: DrawableType::Tileset,
                            thingies: vec![val.clone(), tile.clone()],
                        },
                    );
                }
                _ => panic!("Illegal drawable type {t}!"),
            }
        }
    }

    panic!("Illegal Drawable setup! Expected name, type!")
}
