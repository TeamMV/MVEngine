use ui_parsing::xml::{Entity, XmlValue};

pub struct ParsedAnimation {
    pub(crate) tileset: String,
    pub(crate) range: String,
    pub(crate) fps: u16,
    pub(crate) use_mv: bool,
}

pub fn parse_animation(entity: &Entity) -> (String, ParsedAnimation) {
    if entity.name().as_str() != "animation" {
        panic!(
            "Animation resource must be named animation, got {}!",
            entity.name()
        );
    }

    let tileset = entity.get_attrib("tileset");
    let name = entity.get_attrib("name");
    let fps = entity.get_attrib("fps");

    if let (Some(XmlValue::Str(tileset)), Some(XmlValue::Str(name)), Some(XmlValue::Str(fps))) =
        (tileset, name, fps)
    {
        let range = if let Some(XmlValue::Str(range)) = entity.get_attrib("range") {
            range.to_string()
        } else {
            "..".to_string()
        };

        let use_mv = if let Some(XmlValue::Str(mv)) = entity.get_attrib("mv") {
            !mv.is_empty()
        } else {
            false
        };

        (
            name.to_string(),
            ParsedAnimation {
                tileset: tileset.to_string(),
                range,
                fps: fps
                    .parse::<u16>()
                    .expect("fps must be a positive number between 0-65535"),
                use_mv,
            },
        )
    } else {
        panic!("Animation must contain 'tileset', 'name' and 'fps' attributes");
    }
}
