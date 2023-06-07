use std::path::{Path, PathBuf};
use include_dir::Dir;

pub struct UnloadedBundle; //for now
impl UnloadedBundle {
    fn new() -> Self {
        UnloadedBundle
    }
}

pub struct ResourceBundleBuilder {
    bundle: UnloadedBundle
}

impl ResourceBundleBuilder {
    pub fn new() -> Self {
        ResourceBundleBuilder {
            bundle: UnloadedBundle::new()
        }
    }

    pub fn load_static(self, dir: Dir<'static>) {

    }

    pub fn load_dynamic(self, path: PathBuf) {

    }
}

impl Default for ResourceBundleBuilder {
    fn default() -> Self {
        ResourceBundleBuilder::new()
    }
}