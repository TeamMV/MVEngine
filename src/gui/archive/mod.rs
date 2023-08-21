use mvutils::save::{Loader, Savable, Saver};

pub mod compress;

pub(crate) struct GRes {}

impl Savable for GRes {
    fn save(&self, saver: &mut impl Saver) {
        todo!()
    }

    fn load(loader: &mut impl Loader) -> Result<Self, String> {
        todo!()
    }
}

impl GRes {}

pub struct Error {
    msg: String,
    file: String,
}
