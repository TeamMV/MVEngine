use mvutils::hashers::U32IdentityHasher;
use mvutils::Savable;

#[derive(Savable, Debug, Default)]
pub struct AtlasData {
    pub(crate) atlas: Atlas,
    pub(crate) metrics: Metrics,
    pub(crate) glyphs: Vec<Glyph>,
    pub(crate) kerning: Vec<Kerning>,
}

#[derive(Debug)]
pub struct PreparedAtlasData {
    pub(crate) atlas: Atlas,
    pub(crate) metrics: Metrics,
    pub(crate) glyphs: hashbrown::HashMap<u32, Glyph, U32IdentityHasher>,
    pub(crate) kerning: hashbrown::HashMap<u32, Vec<(u32, f64)>, U32IdentityHasher>,
}

#[derive(Savable, Debug, Default)]
pub(crate) struct Atlas {
    pub(crate) distance_range: f64,
    pub(crate) distance_range_middle: f64,
    pub(crate) size: f64,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) y_origin: YOrigin,
}

#[derive(Savable, Debug, Default)]
pub(crate) enum YOrigin {
    Top,
    #[default]
    Bottom
}

#[derive(Savable, Debug, Default)]
pub(crate) struct Metrics {
    pub(crate) em_size: f64,
    pub(crate) line_height: f64,
    pub(crate) ascender: f64,
    pub(crate) descender: f64,
    pub(crate) underline_y: f64,
    pub(crate) underline_thickness: f64,
}

#[derive(Savable, Debug, Default)]
pub(crate) struct Glyph {
    pub(crate) unicode: u32,
    pub(crate) advance: f64,
    pub(crate) plane_bounds: Bounds,
    pub(crate) atlas_bounds: Bounds
}

#[derive(Savable, Debug, Default)]
pub(crate) struct Bounds {
    pub(crate) left: f64,
    pub(crate) bottom: f64,
    pub(crate) right: f64,
    pub(crate) top: f64,
}

#[derive(Savable, Debug, Default)]
pub(crate) struct Kerning {
    pub(crate) first: u32,
    pub(crate) second: u32,
    pub(crate) kerning: f64,
}

impl Into<PreparedAtlasData> for AtlasData {
    fn into(self) -> PreparedAtlasData {
        let mut glyphs = hashbrown::HashMap::with_capacity_and_hasher(self.glyphs.len(), U32IdentityHasher::default());
        let mut kerning: hashbrown::HashMap<u32, Vec<(u32, f64)>, U32IdentityHasher> = hashbrown::HashMap::with_capacity_and_hasher(self.kerning.len(), U32IdentityHasher::default());

        for glyph in self.glyphs {
            glyphs.insert(glyph.unicode, glyph);
        }

        for kern in self.kerning {
            kerning.entry(kern.first).or_default().push((kern.second, kern.kerning));
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
    pub(crate) fn find_glyph(&self, c: char) -> Option<&Glyph> {
        self.glyphs.get(&(c as u32))
    }

    pub(crate) fn get_kerning(&self, first: char, second: char) -> Option<f64> {
        let kernings = self.kerning.get(&(first as u32))?;
        let index = kernings.binary_search_by(|(c, _)| c.cmp(&(second as u32))).ok()?;
        Some(kernings[index].1)
    }
}