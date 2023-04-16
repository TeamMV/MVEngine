use std::rc::Rc;
use include_dir::{Dir, include_dir};
use crate::assets::{AssetManager, SemiAutomaticAssetManager};
use crate::render::{RenderCore, RenderingBackend};

pub mod assets;
pub mod render;
pub mod parser;

pub struct MVCore {
    assets: Rc<SemiAutomaticAssetManager>,
    render: Option<RenderCore>
}

impl MVCore {
    pub fn new() -> MVCore {
        static DIR: Dir = include_dir!("assets");
        let assets = Rc::new(AssetManager::semi_automatic(DIR.clone()));
        MVCore {
            assets,
            render: None
        }
    }

    pub fn init_render(&mut self, backend: RenderingBackend) {
        self.render = Some(RenderCore::new(backend, self.assets.clone()));
    }
}

#[cfg(test)]
mod tests {
    use cgmath::{Vector4, Zero};

    use crate::render::RenderCore;
    use crate::render::RenderingBackend::OpenGL;
    use crate::render::shared::*;

    #[test]
    fn it_works() {
        let mut renderer = RenderCore::new(OpenGL);
        let mut info = WindowCreateInfo::default();
        info.title = "MVCore".to_string();
        let mut window = renderer.create_window(info);
        window.run_default();
    }
}
