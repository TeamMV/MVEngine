use mvutils::utils::Recover;
use std::sync::Arc;

use mvutils::version::Version;

use mvcore::render::color::RgbColor;
use mvcore::render::window::{Window, WindowSpecs};
use mvcore::render::ApplicationLoopCallbacks;
use mvcore::user_input::input;
use mvcore::{ApplicationInfo, MVCore};

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
    specs.width = 800;
    specs.height = 800;
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
            ctx.rotate(90f32);
            //ctx.scale(1.0f32 / 2.0f32.sqrt(), 1.0f32 / 2.0f32.sqrt());
            ctx.origin(window.specs.get().width as f32 / 2.0, window.specs.get().height as f32 / 2.0);
            ctx.rectangle(input.positions[0], input.positions[1], 100, 100);
            ctx.void_rectangle(0, 0, window.specs.get().width as i32, window.specs.get().height as i32, 2);
            ctx.reset_transformations();
            ctx.color(RgbColor::blue());
            ctx.rectangle(input.positions[0], input.positions[1], 100, 100);
            ctx.color(RgbColor::white());
            let mut t = char::from_u32(1168).unwrap().to_string();
            t.push(char::from_u32(1280).unwrap());
            ctx.text(false, 0, 300, 100, "AV");
        });
    }

    fn effect(&self, window: Arc<Window<Self>>) {
        //window.enable_effect_2d("pixelate".to_string());
        //window.enable_effect_2d("blur".to_string());
        //window.enable_effect_2d("distort".to_string());
        //window.enable_effect_2d("wave".to_string());
    }

    fn exit(&self, window: Arc<Window<Self>>) {}
}
