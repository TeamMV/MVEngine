use mvutils::utils::Recover;
use std::sync::Arc;

use mvutils::version::Version;

use mvcore::render::color::RgbColor;
use mvcore::render::window::{Window, WindowSpecs};
use mvcore::render::ApplicationLoopCallbacks;
use mvcore::user_input::input;
use mvcore::{ApplicationInfo, MVCore};

fn main() {
    env_logger::init();
    let core = MVCore::new(ApplicationInfo {
        name: "Test".to_string(),
        version: Version::new(1, 0, 0),
        multithreaded: true,
        extra_threads: 1,
    });
    let mut specs = WindowSpecs::default();
    specs.vsync = false;
    specs.fps = 60;
    specs.decorated = true;
    specs.resizable = true;
    specs.width = 800;
    specs.height = 600;
    core.get_render().run_window(specs, ApplicationLoop);
}

struct ApplicationLoop;

impl ApplicationLoopCallbacks for ApplicationLoop {
    fn start(&self, window: Arc<Window<Self>>) {}

    fn update(&self, window: Arc<Window<Self>>) {}

    fn draw(&self, window: Arc<Window<Self>>) {
        let tmp = window.input();
        let input = tmp.read().recover();

        window.draw_2d_pass(|ctx| {
            if input.keys[input::KEY_C] {
                ctx.color(RgbColor::red());
            } else {
                ctx.color(RgbColor::green());
            }
            ctx.rectangle(input.positions[0], input.positions[1], 100, 100);
        });
    }

    fn effect(&self, window: Arc<Window<Self>>) {
        //window.enable_effect_2d("blur".to_string());
        //window.enable_effect_2d("pixelate".to_string());
        //window.enable_effect_2d("distort".to_string());
        //window.enable_effect_2d("wave".to_string());
    }

    fn exit(&self, window: Arc<Window<Self>>) {}
}
