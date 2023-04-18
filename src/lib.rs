extern crate alloc;

use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::rc::Rc;
use include_dir::{Dir, include_dir};
use crate::assets::{AssetManager, SemiAutomaticAssetManager, WritableAssetManager};
use crate::render::{RenderCore, RenderingBackend};

pub mod assets;
pub mod render;
pub mod parser;

pub struct MVCore {
    assets: Rc<RefCell<SemiAutomaticAssetManager>>,
    render: Option<RenderCore>
}

impl MVCore {
    pub fn new() -> MVCore {
        static DIR: Dir = include_dir!("assets");
        let assets = Rc::new(RefCell::new(AssetManager::semi_automatic(DIR.clone())));
        MVCore{
            assets,
            render: None
        }
    }

    pub fn init_render(&mut self, backend: RenderingBackend) {
        self.render = Some(RenderCore::new(backend, self.assets.clone()));
        self.assets.borrow_mut().load_shader(self.get_render(), "default", "shaders/default.vert", "shaders/default.frag");
        self.assets.borrow_mut().load_effect_shader(self.get_render(), "blur","shaders/blur.frag");
        self.assets.borrow_mut().load_effect_shader(self.get_render(), "pixelate","shaders/pixelate.frag");
    }

    pub fn get_render(&self) -> &RenderCore {
        self.render.as_ref().expect("RenderCore not initialized!")
    }

    pub fn get_asset_manager(&self) -> Ref<SemiAutomaticAssetManager> {
        self.assets.borrow()
    }
}

#[cfg(test)]
mod tests {
    use crate::assets::ReadableAssetManager;
    use crate::MVCore;

    use crate::render::{RenderCore, RenderingBackend};
    use crate::render::RenderingBackend::OpenGL;
    use crate::render::shared::*;

    #[test]
    fn it_works() {
        let mut core = MVCore::new();
        core.init_render(OpenGL);
        let mut render = core.get_render();
        let mut info = WindowCreateInfo::default();
        info.fps = 10000;
        info.title = "MVCore".to_string();
        info.fullscreen = false;
        info.width = 1920;
        info.height = 1080;
        let mut window = render.create_window(info);
        window.add_shader("blur", core.get_asset_manager().get_effect_shader("blur"));
        window.add_shader("pixelate", core.get_asset_manager().get_effect_shader("pixelate"));
        window.run(Test);
        let a = vec![1];
    }

    struct Test;

    impl ApplicationLoop for Test {
        fn start(&self, window: &mut impl Window) {

        }

        fn update(&self, window: &mut impl Window) {

        }

        fn draw(&self, window: &mut impl Window) {
            window.get_draw_2d().tri();
            window.queue_shader_pass(ShaderPassInfo::new("pixelate", |shader| {
              shader.uniform_1f("size", 10.0);
            }));
            window.queue_shader_pass(ShaderPassInfo::new("blur", |shader| {
                shader.uniform_1f("dir", 16.0);
                shader.uniform_1f("quality", 4.0);
                shader.uniform_1f("size", 8.0);
            }));
        }

        fn stop(&self, window: &mut impl Window) {

        }
    }
}
