use ui_parsing::xml::{Entity, XmlValue};

pub enum PartRes {
    Texture(String),
    Anim(String),
    TileSet(String, String)
}

pub struct ParsedComposite {
    pub(crate) rig: String,
    pub(crate) parts: Vec<PartRes>
}

pub fn parse_composite(entity: &Entity) -> (String, ParsedComposite) {
    if entity.name().as_str() != "composite" {
        panic!("Composite resource must be named animation, got {}!", entity.name());
    }

    let name = entity.get_attrib("name");
    let rig = entity.get_attrib("rig");

    if let (Some(XmlValue::Str(name)), Some(XmlValue::Str(rig))) = (name, rig) {
        let mut parts = vec![];
        if let Some(XmlValue::Entities(children)) = entity.inner() {
            for child in children {
                if child.name() == "part" {
                    let res = child.get_attrib("res");
                    if let Some(XmlValue::Str(res)) = res {
                        let (beginning, rem) = res.split_once('.').expect("Invalid resource!");
                        let p_res =  match beginning {
                            "texture" => PartRes::Texture(res.to_string()),
                            "animation" => PartRes::Anim(res.to_string()),
                            "tile" => {
                                let (ts, _) = rem.split_once('.').expect("Expected valid tileset");
                                PartRes::TileSet(ts.to_string(), res.to_string())
                            },
                            _ => panic!("Unsupported resource")
                        };
                        parts.push(p_res);
                    }
                }
            }
        }
        (name.to_string(), ParsedComposite {
            rig: rig.to_string(),
            parts,
        })
    } else {
        panic!("Animation must contain 'tileset', 'name' and 'fps' attributes");
    }
}