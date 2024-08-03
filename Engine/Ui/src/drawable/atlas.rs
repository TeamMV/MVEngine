use crate::resources::resources::R;

pub struct BoxSheet {
    specs: BoxSheetSpecs,
    texture: String
}

impl BoxSheet {
    pub fn new(specs: BoxSheetSpecs, texture: String) -> Self {
        Self { specs, texture }
    }

    pub fn init(&mut self) {
        let tex = R::texture_regions().get(&self.texture);
    }
}

pub struct BoxSheetSpecs {

}