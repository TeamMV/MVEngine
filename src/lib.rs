extern crate alloc;

use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use include_dir::{Dir, include_dir};
use mvutils::version::Version;

use crate::assets::{AssetManager, SemiAutomaticAssetManager, WritableAssetManager};
use crate::render::{RenderCore, RenderingBackend};

pub mod assets;
pub mod render;
pub mod parser;

pub struct MVCore {
    assets: Rc<RefCell<SemiAutomaticAssetManager>>,
    render: Rc<RenderCore>,
    app_version: Version
}

impl MVCore {
    pub fn new(backend: RenderingBackend, app_version: Version) -> MVCore {
        static DIR: Dir = include_dir!("assets");
        let assets = Rc::new(RefCell::new(AssetManager::semi_automatic(DIR.clone())));
        let render = Rc::new(RenderCore::new(backend, assets.clone()));
        assets.borrow_mut().set_render_core(render.clone());
        MVCore {
            assets,
            render,
            app_version
        }.tmp_load()
    }

    pub fn tmp_load(self) -> Self {
        self.assets.borrow_mut().load_bitmap_font("default", "fonts/font.png", "fonts/default.fnt");
        self.assets.borrow_mut().load_shader("default", "shaders/default.vert", "shaders/default.frag");
        self.assets.borrow_mut().load_effect_shader("blur", "shaders/blur.frag");
        self.assets.borrow_mut().load_effect_shader("pixelate", "shaders/pixelate.frag");
        self
    }

    pub fn get_app_version(&self) -> Version {
        self.app_version
    }

    pub fn get_render(&self) -> Rc<RenderCore> {
        self.render.clone()
    }

    pub fn get_asset_manager(&self) -> Ref<SemiAutomaticAssetManager> {
        self.assets.borrow()
    }

    pub fn terminate(mut self) {
        self.term();
        drop(self);
    }

    fn term(&mut self) {
        self.render.terminate();
    }
}

impl Drop for MVCore {
    fn drop(&mut self) {
        self.term();
    }
}

impl Default for MVCore {
    fn default() -> Self {
        Self::new(RenderingBackend::OpenGL, Version::default())
    }
}

#[cfg(test)]
//#[cfg(not(feature = "vulkan"))]
mod tests {
    use crate::assets::ReadableAssetManager;
    use crate::MVCore;
    use crate::render::RenderingBackend::OpenGL;
    use crate::render::shared::*;
    use mvutils::version::Version;

    #[test]
    fn test() {
        let mut core = MVCore::new(OpenGL, Version::parse("v0.1.0").unwrap());
        let mut info = WindowCreateInfo::default();
        info.title = "MVCore".to_string();
        let mut window = core.get_render().create_window(info);
        window.add_shader("blur", core.get_asset_manager().get_effect_shader("blur"));
        window.add_shader("pixelate", core.get_asset_manager().get_effect_shader("pixelate"));
        window.run(Test { core });
    }

    struct Test {
        core: MVCore
    }

    impl ApplicationLoop for Test {
        fn start(&self, window: &mut impl Window) {}

        fn update(&self, window: &mut impl Window) {}

        fn draw(&self, window: &mut impl Window) {
            //window.get_draw_2d().tri();
            window.get_draw_2d().text(true, 100, 100, 20, "Hello".to_string());
            window.get_draw_2d().rectangle(100, 130, self.core.get_asset_manager().get_font("default").get_metrics("Hello").width(20), 30);
            //window.queue_shader_pass(ShaderPassInfo::new("pixelate", |shader| {
            //  shader.uniform_1f("size", 10.0);
            //}));
            //window.queue_shader_pass(ShaderPassInfo::new("blur", |shader| {
            //    shader.uniform_1f("dir", 16.0);
            //    shader.uniform_1f("quality", 4.0);
            //    shader.uniform_1f("size", 8.0);
            //}));
        }

        fn stop(&self, window: &mut impl Window) {}
    }
}

#[cfg(test)]
#[cfg(feature = "vulkan")]
mod vulkan_test {


    #[test]
    fn test() {

    }

}
