extern crate alloc;
extern crate core;

use std::cell::{Ref, RefCell};
use std::rc::Rc;

use include_dir::{Dir, include_dir};
use mvutils::version::Version;

use crate::assets::{AssetManager, SemiAutomaticAssetManager};
use crate::render::{RenderCore, RenderingBackend};

pub mod assets;
pub mod render;
pub mod parser;
pub mod input;
#[cfg(feature = "gui")]
pub mod gui;

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
#[cfg(feature = "gui")]
mod tests {
    use std::cell::RefCell;
    use std::ops::Deref;
    use mvutils::screen::Measurement;
    use mvutils::utils::RcMut;
    use crate::assets::ReadableAssetManager;
    use crate::{ApplicationInfo, MVCore, resolve, setup};
    use crate::render::RenderingBackend::OpenGL;
    use crate::render::shared::*;
    use mvutils::version::Version;
    use crate::gui::components::{GuiElementAbs, GuiMarkdown, GuiTextComponent};
    use crate::gui::gui_formats::FormattedString;
    use crate::gui::styles::{BorderStyle, GuiValue};
    use crate::gui::styles::Positioning::Absolute;
    use crate::render::color::{Color, Gradient, RGB};
    use crate::render::text::TypeFace;

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
        window.run(Test {
            core,
            md: GuiMarkdown::create()
        });
    }

    struct Test {
        core: MVCore,
        md: GuiMarkdown
    }

    impl ApplicationLoop for Test {
        fn start(&mut self, mut window: RunningWindow) {
            self.md.set_text(FormattedString::new("Hello"));

            setup!(
                self.md.info_mut().style => {
                    font: (Some(TypeFace::single(self.core.assets.borrow_mut().get_font("default")))),
                    text_size: 2 cm,
                    text_color: (Gradient::new(Color::<RGB, f32>::black())),
                    text_chroma: true,
                    background_color: (Gradient::new(Color::<RGB, f32>::cyan())),
                    border_width: 5,
                    border_style: (BorderStyle::Square),
                    border_radius: 20,
                    border_color: (Gradient::new(Color::<RGB, f32>::red())),
                    x: (window.get_width() - 100),
                    y: 100,
                    origin: (Origin::BottomLeft),
                    position: Absolute
                }
            );

            //self.md.info_mut().style.padding_top = GuiValue::Just(20);
            //self.md.info_mut().style.padding_bottom = GuiValue::Just(20);
            //self.md.info_mut().style.padding_right = GuiValue::Just(20);
            //self.md.info_mut().style.padding_left = GuiValue::Just(20);
        }

        fn update(&mut self, mut window: RunningWindow) {

        }

        fn draw(&mut self, mut window: RunningWindow) {
            //window.get_draw_2d().tri();
            //window.get_draw_2d().text(true, 100, 100, 50, "Hello");
            //window.get_draw_2d().rectangle(100, 150, self.core.get_asset_manager().get_font("default").get_metrics("Hello").width(50), 50);
            //window.queue_shader_pass(ShaderPassInfo::new("pixelate", |shader| {
            //  shader.uniform_1f("size", 10.0);
            //}));
            //window.queue_shader_pass(ShaderPassInfo::new("blur", |shader| {
            //    shader.uniform_1f("dir", 16.0);
            //    shader.uniform_1f("quality", 4.0);
            //    shader.uniform_1f("size", 8.0);
            //}));
            self.md.draw(window.get_draw_2d());
        }

        fn stop(&mut self, window: RunningWindow) {}
    }
}