use mvutils::once::CreateOnce;
use mvutils::unsafe_utils::DangerousCell;
use mvutils::utils::{Map, Recover};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use mvutils::version::Version;

use mvcore::render::color::RgbColor;
use mvcore::render::common::TextureRegion;
use mvcore::render::window::{Window, WindowSpecs};
use mvcore::render::ApplicationLoopCallbacks;
use mvcore::ui::ease;
use mvcore::ui::ease::Easing;
use mvcore::ui::elements::UiElementImpl;
use mvcore::ui::prelude::{
    Background, Position, RectangleBackground, RoundedBackground, UiElement, UiElementCallbacks,
    UiValue,
};
use mvcore::ui::styles::Dimension;
#[cfg(feature = "ui")]
use mvcore::ui::timing::{DurationTask, TimingManager};
use mvcore::{input, ApplicationInfo, MVCore};

fn main() {
    let core = MVCore::new(ApplicationInfo {
        name: "Test".to_string(),
        version: Version::new(1, 0, 0),
        multithreaded: true,
        extra_threads: 1,
    });
    let mut specs = WindowSpecs::default();
    specs.vsync = false;
    specs.fps = 1;
    specs.decorated = true;
    specs.resizable = true;
    specs.transparent = false;
    specs.width = 800;
    specs.height = 600;
    core.get_render().run_window(
        specs,
        ApplicationLoop {
            tex: CreateOnce::new(),
        },
    );
}

struct ApplicationLoop {
    tex: CreateOnce<Arc<TextureRegion>>,
}

impl ApplicationLoopCallbacks for ApplicationLoop {
    fn start(&self, window: Arc<Window<Self>>) {
        self.tex.create(|| {
            Arc::new(TextureRegion::from(Arc::new(
                window.create_texture(include_bytes!("cursor.png").to_vec()),
            )))
        });
    }

    fn update(&self, window: Arc<Window<Self>>) {}

    fn draw(&self, window: Arc<Window<Self>>) {
        let binding = window.input();
        let input = binding.read().recover();

        let bg = RoundedBackground::new(Dimension::new(100, 50));
        let mut elem = UiElementImpl::test();
        let style = elem.style_mut();
        style.background.main_color = UiValue::Just(RgbColor::blue());
        style.background.border_color = UiValue::Just(RgbColor::white());
        style.background.border_width = UiValue::Just(2);

        window.draw_2d_pass(|ctx| unsafe {
            elem.compute_values(ctx);
            elem.draw(ctx);
            bg.draw(ctx, Arc::new(RwLock::new(elem)));
        });
    }

    fn effect(&self, window: Arc<Window<Self>>) {
        //window.enable_effect_2d("wave".to_string());
        //window.enable_effect_2d("pixelate".to_string());
        //window.enable_effect_2d("blur".to_string());
        //window.enable_effect_2d("distort".to_string());
    }

    fn exit(&self, window: Arc<Window<Self>>) {}
}
