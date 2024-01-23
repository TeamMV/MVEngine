use mvutils::once::CreateOnce;
use mvutils::utils::Recover;
use std::sync::Arc;

use mvutils::version::Version;

use mvcore::render::color::RgbColor;
use mvcore::render::common::TextureRegion;
use mvcore::render::window::{Cursor, Window, WindowSpecs};
use mvcore::render::ApplicationLoopCallbacks;
use mvcore::{ApplicationInfo, input, MVCore};

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
        window.set_cursor(Cursor::SoftBusy);
    }

    fn update(&self, window: Arc<Window<Self>>) {}

    fn draw(&self, window: Arc<Window<Self>>) {
        let tmp = window.input();
        let input = tmp.read().recover();

        window.draw_2d_pass(|ctx| {
            if input.keys[input::KEY_C] {
                ctx.color(RgbColor::red());
                window.set_cursor(Cursor::None);
            } else {
                ctx.color(RgbColor::green());
            }

            if input.keystates[input::KEY_R] == input::State::JustPressed {
                window.restore_cursor();
            }

            ctx.rectangle(100, 100, 100, 100);
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
