use ui_parsing::xml::{Entity, XmlValue};

pub struct ParsedComposite {
    pub(crate) rig: String,
    pub(crate) parts: Vec<CompositePart>
}

pub struct CompositePart {
    pub(crate) name: String,
    pub(crate) res: String
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
                        if let Some(XmlValue::Str(name)) = child.get_attrib("name") {
                            parts.push(CompositePart {
                                name: name.clone(),
                                res: res.clone(),
                            });
                        }
                    }
                }
            }
        }
        (name.to_string(), ParsedComposite {
            rig: rig.to_string(),
            parts,
        })
    } else {
        panic!("Illegal Composite setup!");
    }
}