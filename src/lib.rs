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
#[cfg(feature = "3d")]
mod tests {
    use std::cell::RefCell;
    use std::ops::{Deref, DerefMut};
    use std::sync::{Arc, RwLock};
    use glam::{Mat4, Vec3};
    use mvutils::screen::Measurement;
    use mvutils::utils::RcMut;
    use crate::assets::{ReadableAssetManager, WritableAssetManager};
    use crate::{ApplicationInfo, MVCore, resolve, setup};
    use crate::render::RenderingBackend::OpenGL;
    use crate::render::shared::*;
    use mvutils::version::Version;
    use crate::gui::components::{GuiElementAbs, GuiMarkdown, GuiTextComponent};
    use crate::gui::gui_formats::FormattedString;
    use crate::gui::styles::{BorderStyle, GuiValue, Origin};
    use crate::gui::styles::Positioning::Absolute;
    use crate::render::batch3d::BatchController3D;
    use crate::render::camera::Camera3D;
    use crate::render::color::{Color, Gradient, RGB};
    use crate::render::lights::Light;
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
            md: GuiMarkdown::create(),
            batch: None,
            demo_lights: Vec::new()
        });
    }

    struct Test {
        core: MVCore,
        md: GuiMarkdown,
        batch: Option<BatchController3D>,
        demo_lights: Vec<Light>
    }

    impl ApplicationLoop for Test {
        fn start(&mut self, mut window: RunningWindow) {
            self.core.assets.borrow_mut().load_model("figcolor", "models/figcolor.obj");

            let shader = self.core.assets.borrow().get_shader("model");
            let s = self.core.assets.borrow().get_shader("batch");
            self.batch = Some(BatchController3D::new(s, shader, 10000));

            self.demo_lights = vec![Light {
                position: Vec3::new(0.0, 0.0, 5.0),
                direcetion: Vec3::new(0.5, 0.0, 0.5),
                color: Color::<RGB, f32>::white(),
                attenuation: 0.0,
                cutoff: 0.0,
                radius: 100.0,
            }];

            let cam = window.get_camera_3d();

            unsafe {
                let cam = (cam as *const Camera3D).cast_mut().as_mut().unwrap();
                cam.rotation = Vec3::new(0.0, 1.0, 0.0);
            }

            //self.md.set_text(FormattedString::new("Hello"));
            //setup!(
            //    self.md.info_mut().style => {
            //        font: (Some(TypeFace::single(self.core.assets.borrow_mut().get_font("default")))),
            //        text_size: 2 cm,
            //        text_color: (Gradient::new(Color::<RGB, f32>::black())),
            //        text_chroma: true,
            //        background_color: (Gradient::new(Color::<RGB, f32>::cyan())),
            //        border_width: 5,
            //        border_style: (BorderStyle::Square),
            //        border_radius: 20,
            //        border_color: (Gradient::new(Color::<RGB, f32>::red())),
            //        x: (window.get_width() - 100),
            //        y: 100,
            //        origin: (Origin::BottomLeft),
            //        position: Absolute
            //    }
            //);

            //self.md.info_mut().style.padding_top = GuiValue::Just(20);
            //self.md.info_mut().style.padding_bottom = GuiValue::Just(20);
            //self.md.info_mut().style.padding_right = GuiValue::Just(20);
            //self.md.info_mut().style.padding_left = GuiValue::Just(20);
        }

        fn update(&mut self, mut window: RunningWindow) {

        }

        fn draw(&mut self, mut window: RunningWindow) {
            let model = self.core.assets.borrow().get_model("figcolor");
            self.batch.as_mut().unwrap().add_model(model, [0.0, 0.0, 800.0, 600.0, 0.0, 0.0], Mat4::IDENTITY);
            if let RunningWindow::OpenGL(w) = window {
                unsafe {
                    let w = w.as_mut().unwrap();
                    self.batch.as_mut().unwrap().render(w.get_render_3d(), window.get_camera_3d());
                    let shader = self.core.assets.borrow().get_effect_shader("deferred");
                    w.get_lighting().light_scene(shader, &w.get_render_3d().buffer, w.get_camera_3d(), &self.demo_lights);
                }
            }
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
            //self.md.draw(window.get_draw_2d());
        }

        fn stop(&mut self, window: RunningWindow) {}
    }
}