use std::sync::Arc;

use mvcore::ApplicationLoopCallbacks;
use mvcore::render::RenderCore;
use mvcore::render::window::{Window, WindowSpecs};

fn main() {
    env_logger::init();
    let core = RenderCore::new();
    let mut specs = WindowSpecs::default();
    specs.vsync = false;
    specs.fps = 10000;
    specs.decorated = true;
    specs.resizable = true;
    specs.width = 800;
    specs.height = 600;
    core.run_window(specs, ApplicationLoop);
}

struct ApplicationLoop;
impl ApplicationLoopCallbacks for ApplicationLoop {
    fn start(&self, window: Arc<Window<Self>>) {

    }

    fn update(&self, window: Arc<Window<Self>>) {

    }

    fn draw(&self, window: Arc<Window<Self>>) {

    }

    fn exit(&self, window: Arc<Window<Self>>) {

    }
}