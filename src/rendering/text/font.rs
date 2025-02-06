use mvutils::hashers::U32IdentityHasher;
use mvutils::Savable;

#[derive(Savable, Debug, Default)]
pub struct AtlasData {
    pub atlas: Atlas,
    pub metrics: Metrics,
    pub glyphs: Vec<Glyph>,
    pub kerning: Vec<Kerning>,
}

#[derive(Debug)]
pub struct PreparedAtlasData {
    pub atlas: Atlas,
    pub metrics: Metrics,
    pub glyphs: hashbrown::HashMap<u32, Glyph, U32IdentityHasher>,
    pub kerning: hashbrown::HashMap<u32, Vec<(u32, f64)>, U32IdentityHasher>,
}

#[derive(Savable, Debug, Default)]
pub struct Atlas {
    pub distance_range: f64,
    pub distance_range_middle: f64,
    pub size: f64,
    pub width: u32,
    pub height: u32,
    pub y_origin: YOrigin,
}

#[derive(Savable, Debug, Default)]
pub enum YOrigin {
    Top,
    #[default]
    Bottom,
}

#[derive(Savable, Debug, Default)]
pub struct Metrics {
    pub em_size: f64,
    pub line_height: f64,
    pub ascender: f64,
    pub descender: f64,
    pub underline_y: f64,
    pub underline_thickness: f64,
}

#[derive(Savable, Debug, Default)]
pub struct Glyph {
    pub unicode: u32,
    pub advance: f64,
    pub plane_bounds: Bounds,
    pub atlas_bounds: Bounds,
}

#[derive(Savable, Debug, Default)]
pub struct Bounds {
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
    pub top: f64,
}

#[derive(Savable, Debug, Default)]
pub struct Kerning {
    pub first: u32,
    pub second: u32,
    pub kerning: f64,
}

impl Into<PreparedAtlasData> for AtlasData {
    fn into(self) -> PreparedAtlasData {
        let mut glyphs = hashbrown::HashMap::with_capacity_and_hasher(
            self.glyphs.len(),
            U32IdentityHasher::default(),
        );
        let mut kerning: hashbrown::HashMap<u32, Vec<(u32, f64)>, U32IdentityHasher> =
            hashbrown::HashMap::with_capacity_and_hasher(
                self.kerning.len(),
                U32IdentityHasher::default(),
            );

        for glyph in self.glyphs {
            glyphs.insert(glyph.unicode, glyph);
        }

        for kern in self.kerning {
            kerning
                .entry(kern.first)
                .or_default()
                .push((kern.second, kern.kerning));
        }

        glyphs.shrink_to_fit();
        kerning.shrink_to_fit();

        PreparedAtlasData {
            atlas: self.atlas,
            metrics: self.metrics,
            glyphs,
            kerning,
        }
    }
}

impl PreparedAtlasData {
    pub fn find_glyph(&self, c: char) -> Option<&Glyph> {
        self.glyphs.get(&(c as u32))
    }

    pub fn get_kerning(&self, first: char, second: char) -> Option<f64> {
        let kernings = self.kerning.get(&(first as u32))?;
        let index = kernings
            .binary_search_by(|(c, _)| c.cmp(&(second as u32)))
            .ok()?;
        Some(kernings[index].1)
    }
}
