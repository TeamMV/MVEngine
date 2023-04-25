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
pub mod input;
#[cfg(feature = "gui")]
pub mod vgui_v5;

pub struct MVCore {
    assets: Rc<RefCell<SemiAutomaticAssetManager>>,
    render: Rc<RenderCore>,
    info: ApplicationInfo
}

impl MVCore {
    pub fn new(info: ApplicationInfo) -> MVCore {
        static DIR: Dir = include_dir!("assets");
        let assets = Rc::new(RefCell::new(AssetManager::semi_automatic(DIR.clone())));
        let render = Rc::new(RenderCore::new(&info, assets.clone()));
        assets.borrow_mut().set_render_core(render.clone());
        MVCore {
            assets,
            render,
            info
        }
    }

    pub fn get_app_version(&self) -> Version {
        self.info.version
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
        Self::new(ApplicationInfo::default())
    }
}

pub struct ApplicationInfo {
    name: String,
    version: Version,
    backend: RenderingBackend
}

impl ApplicationInfo {
    fn new(backend: RenderingBackend, name: &str, version: Version) -> ApplicationInfo {
        ApplicationInfo {
            name: name.to_string(),
            version,
            backend
        }
    }
}

impl Default for ApplicationInfo {
    fn default() -> Self {
        ApplicationInfo {
            name: String::new(),
            version: Version::default(),
            backend: RenderingBackend::OpenGL
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assets::ReadableAssetManager;
    use crate::{ApplicationInfo, MVCore};
    use crate::render::RenderingBackend::OpenGL;
    use crate::render::shared::*;
    use mvutils::version::Version;

    #[test]
    fn test() {
        let mut app = ApplicationInfo::default();
        app.version = Version::parse("v0.1.0").unwrap();
        app.name = "Test".to_string();
        app.backend = OpenGL;
        let mut core = MVCore::new(app);
        let mut info = WindowCreateInfo::default();
        info.title = "MVCore".to_string();
        let mut window = core.get_render().create_window(info);
        window.run(Test { core });
    }

    struct Test {
        core: MVCore
    }

    impl ApplicationLoop for Test {
        fn start(&self, mut window: RunningWindow) {}

        fn update(&self, mut window: RunningWindow) {}

        fn draw(&self, mut window: RunningWindow) {
            //window.get_draw_2d().tri();
            window.get_draw_2d().text(true, 100, 100, 50, "Hello".to_string());
            window.get_draw_2d().rectangle(100, 150, self.core.get_asset_manager().get_font("default").get_metrics("Hello").width(50), 50);
            //window.queue_shader_pass(ShaderPassInfo::new("pixelate", |shader| {
            //  shader.uniform_1f("size", 10.0);
            //}));
            //window.queue_shader_pass(ShaderPassInfo::new("blur", |shader| {
            //    shader.uniform_1f("dir", 16.0);
            //    shader.uniform_1f("quality", 4.0);
            //    shader.uniform_1f("size", 8.0);
            //}));
        }

        fn stop(&self, window: RunningWindow) {}
    }
}

#[cfg(test)]
#[cfg(feature = "vulkan")]
mod vulkan_tests {
    use mvutils::version::Version;
    use crate::{ApplicationInfo, MVCore};
    use crate::assets::ReadableAssetManager;
    use crate::render::RenderingBackend::Vulkan;
    use crate::render::shared::{ApplicationLoop, RunningWindow, WindowCreateInfo};

    #[test]
    fn test() {
        let mut app = ApplicationInfo::default();
        app.version = Version::parse("v0.1.0").unwrap();
        app.name = "Test".to_string();
        app.backend = Vulkan;
        let mut core = MVCore::new(app);
        let mut info = WindowCreateInfo::default();
        info.title = "MVCore".to_string();
        let mut window = core.get_render().create_window(info);
        window.run(Test { core });
    }

    struct Test {
        core: MVCore
    }

    impl ApplicationLoop for Test {
        fn start(&self, mut window: RunningWindow) {}

        fn update(&self, mut window: RunningWindow) {}

        fn draw(&self, mut window: RunningWindow) {
            //window.get_draw_2d().tri();
            window.get_draw_2d().text(true, 100, 100, 50, "Hello".to_string());
            window.get_draw_2d().rectangle(100, 150, self.core.get_asset_manager().get_font("default").get_metrics("Hello").width(50), 50);
            //window.queue_shader_pass(ShaderPassInfo::new("pixelate", |shader| {
            //  shader.uniform_1f("size", 10.0);
            //}));
            //window.queue_shader_pass(ShaderPassInfo::new("blur", |shader| {
            //    shader.uniform_1f("dir", 16.0);
            //    shader.uniform_1f("quality", 4.0);
            //    shader.uniform_1f("size", 8.0);
            //}));
        }

        fn stop(&self, window: RunningWindow) {}
    }

}
