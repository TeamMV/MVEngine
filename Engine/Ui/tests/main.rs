use std::sync::Arc;
use log::{error, LevelFilter};
use mvcore::asset::asset::AssetType;
use mvcore::asset::manager::{AssetHandle, AssetManager};
use mvcore::color::parse::parse_color;
use mvcore::color::RgbColor;
use mvcore::render::window::{Window, WindowCreateInfo};
use mvcore::render::ApplicationLoopCallbacks;
use mvcore::render::texture::Texture;
use mvengine_ui::elements::{ComputeUiElement, UiElementCallbacks, UiElementStub};
use mvengine_ui::render::ctx::{DrawContext2D, RectPoint};
use mvengine_ui::render::{ctx, UiRenderer};
use mvengine_ui::timing::{AnimationState, PeriodicTask, TIMING_MANAGER};

use mvengine_ui::uix::UiCompoundElement;

mod test;

fn main() {
    mvlogger::init(std::io::stdout(), LevelFilter::Debug);
    let mut info = WindowCreateInfo::default();
    info.title = "UI test".to_string();
    info.fps = 60;
    info.ups = 20;
    info.vsync = true;

    let window = Window::new(info);
    window.run::<Application>();
    //test::run();
}

struct Application {
    manager: Arc<AssetManager>,
    ctx: DrawContext2D,
    rot: f32,
    img: AssetHandle
}

impl ApplicationLoopCallbacks for Application {
    fn new(window: &mut Window) -> Self {
        let renderer = UiRenderer::new(window, "TestApp".to_string());
        let manager = AssetManager::new(renderer.get_device(), 1);
        let mut ctx = DrawContext2D::new(renderer);

        let img = manager.create_asset("img.png", AssetType::Texture);
        img.load();
        while !img.is_loaded() {

        }

        println!("loaded");
        //unsafe {
        //    TIMING_MANAGER.request(PeriodicTask::new(-1, 1000, |win, _| {
        //        println!("FPS: {}", win.get_value().try_get::<Window>().unwrap().fps());
        //    }, AnimationState::value(window)), None);
        //}

        Self { manager, ctx, rot: 0.0, img }
    }

    fn update(&mut self, window: &mut Window, delta_t: f64) {}

    fn draw(&mut self, window: &mut Window, delta_t: f64) {
        if let Err(e) = self.ctx.draw() {
            error!("{:?}", e);
        }

        self.rot += 0.5;

        unsafe {
            TIMING_MANAGER.post_frame(delta_t as f32, 0);
        }
    }

    fn exiting(&mut self, window: &mut Window) {}

    fn resize(&mut self, window: &mut Window, width: u32, height: u32) {}
}
