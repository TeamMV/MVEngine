use mvutils::once::CreateOnce;
use mvutils::utils::Recover;
use std::sync::{Arc, RwLock};
use mvutils::unsafe_utils::DangerousCell;

use mvutils::version::Version;

use mvcore::render::color::RgbColor;
use mvcore::render::common::TextureRegion;
use mvcore::render::window::{Cursor, Window, WindowSpecs};
use mvcore::render::ApplicationLoopCallbacks;
use mvcore::{input, ApplicationInfo, MVCore};
use mvcore::gui::element_file::{Background, GuiValue, RoundedBackground};
use mvcore::gui::elements::{GuiElement, GuiElementImpl};
use mvcore::gui::styles::Dimension;

fn main() {
    let core = MVCore::new(ApplicationInfo {
        name: "Test".to_string(),
        version: Version::new(1, 0, 0),
        multithreaded: true,
        extra_threads: 1,
    });
    let mut specs = WindowSpecs::default();
    specs.vsync = false;
    specs.fps = 20000;
    specs.decorated = true;
    specs.resizable = true;
    specs.transparent = false;
    specs.width = 800;
    specs.height = 800;
    core.get_render().run_window(
        specs,
        ApplicationLoop {
            tex: CreateOnce::new(),
            elem: Arc::new(RwLock::new(GuiElementImpl::test())),
        },
    );
}

struct ApplicationLoop {
    tex: CreateOnce<Arc<TextureRegion>>,
    elem: Arc<RwLock<GuiElementImpl>>
}

impl ApplicationLoopCallbacks for ApplicationLoop {
    fn start(&self, window: Arc<Window<Self>>) {
        self.tex.create(|| {
            Arc::new(TextureRegion::from(Arc::new(
                window.create_texture(include_bytes!("cursor.png").to_vec()),
            )))
        });
        window.set_cursor(Cursor::SoftBusy);
    }

    fn update(&self, window: Arc<Window<Self>>) {}

    fn draw(&self, window: Arc<Window<Self>>) {
        let tmp = window.input();
        let input = tmp.read().recover();

        let mut g = self.elem.write().recover();

        g.style_mut().background.border_color = GuiValue::Just(RgbColor::white());
        g.style_mut().background.main_color = GuiValue::Just(RgbColor::blue());
        let bg = RoundedBackground::new(Dimension::new(100, 50));

        window.draw_2d_pass(|ctx| {
            //ctx.text_options.kerning = 20.0;
            //ctx.text_options.skew = 20.0;
            //ctx.color(RgbColor::white());
            //ctx.text(false, 100, 100, 200, "Hello");
            //ctx.color(RgbColor::red());
            //ctx.ellipse_arc(200, 200, 200, 100, 90, 0, 200.0);
            g.compute_values(ctx);
            let mut a = self.elem.clone();
            bg.draw(ctx, Arc::new(a.into_inner().unwrap()));
        });

        drop(g);
        drop(bg);
    }

    fn effect(&self, window: Arc<Window<Self>>) {
        //window.enable_effect_2d("wave".to_string());
        //window.enable_effect_2d("pixelate".to_string());
        //window.enable_effect_2d("blur".to_string());
        //window.enable_effect_2d("distort".to_string());
    }

    fn exit(&self, window: Arc<Window<Self>>) {}
}
