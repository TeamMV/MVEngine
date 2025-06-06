use ui_parsing::xml::{Entity, XmlValue};

pub struct ParsedTileSet {
    pub(crate) atlas: String,
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) fps: Option<u16>,
    pub(crate) count: usize,
    pub(crate) linear: bool,
    pub(crate) entries: Vec<(String, usize)>,
}

pub fn parse_tileset(entity: &Entity) -> (String, ParsedTileSet) {
    if entity.name().as_str() != "tileset" {
        panic!("Tileset resource must be named tileset, got {}!", entity.name());
    }

    let atlas = entity.get_attrib("atlas");
    let name = entity.get_attrib("name");
    let width = entity.get_attrib("width");
    let height = entity.get_attrib("height");
    let count = entity.get_attrib("count");

    if let (Some(XmlValue::Str(atlas)), Some(XmlValue::Str(name)), Some(XmlValue::Str(width)), Some(XmlValue::Str(height)), Some(XmlValue::Str(count))) = (atlas, name, width, height, count) {
        let mut linear = false;
        if let Some(XmlValue::Str(sampler)) = entity.get_attrib("sampler") {
            linear = sampler == "linear";
        }

        let fps = if let Some(XmlValue::Str(fps)) = entity.get_attrib("fps") {
            Some(fps.parse::<u16>().expect("fps must be a positive number between 0-65535"))
        } else {
            None
        };

        let mut parsed = ParsedTileSet {
            atlas: atlas.to_string(),
            width: width.parse::<i32>().expect("Tileset width must be a number"),
            height: height.parse::<i32>().expect("Tileset height must be a number"),
            count: count.parse::<usize>().expect("Tileset count must be a number"),
            linear,
            fps,
            entries: vec![],
        };
        if let Some(XmlValue::Entities(entities)) = entity.inner() {
            for entity in entities {
                if entity.name() == "fps" {
                    if let Some(XmlValue::Str(fps)) = entity.get_attrib("value") {
                        parsed.fps = Some(fps.parse::<u16>().expect("fps must be a positive number between 0-65535"));
                    } else if let Some(XmlValue::Str(fps)) = entity.inner() {
                        parsed.fps = Some(fps.parse::<u16>().expect("fps must be a positive number between 0-65535"));
                    } else {
                        panic!("fps tag must either contain 'value' attribute or string child")
                    }
                    continue;
                }
                if entity.name() != "entry" {
                    panic!("Unsupported tileset entry type '{}'", entity.name());
                }
                let name = entity.get_attrib("name");
                let index = entity.get_attrib("index");

                if let (Some(XmlValue::Str(name)), Some(XmlValue::Str(index))) = (name, index) {
                    parsed.entries.push((name.to_string(), index.parse::<usize>().expect("Tileset entry index must be a number")));
                } else {
                    panic!("Tileset entry must contain 'name' and 'index' attributes");
                }
            }
        }

        (name.to_string(), parsed)
    } else {
        panic!("Tileset must contain 'altas', 'name', 'width', 'height' and 'count' attributes");
    }
}