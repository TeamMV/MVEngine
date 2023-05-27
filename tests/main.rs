use log::LevelFilter::Trace;
use mvcore::render::RenderCore;
use mvcore::render::window::{CreatedShader, WindowSpecs};

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
    core.run_window(specs);
}
