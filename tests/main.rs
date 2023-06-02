use std::sync::Arc;
use log::LevelFilter::Trace;
use mvcore::ApplicationLoopCallbacks;
use mvcore::render::RenderCore;
use mvcore::render::window::{CreatedShader, Window, WindowSpecs};

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
    fn start(&self, window: &Window<Self>) {
        todo!()
    }

    fn update(&self, window: &Window<Self>) {
        todo!()
    }

    fn draw(&self, window: &Window<Self>) {
        todo!()
    }

    fn exit(&self, window: &Window<Self>) {
        todo!()
    }
}