use log::LevelFilter::Trace;
use mvcore::render::RenderCore;
use mvcore::render::window::{CreatedShader, WindowSpecs};

fn main() {
    let core = RenderCore::new();
    let mut specs = WindowSpecs::default();
    specs.vsync = false;
    specs.fps = 10000;
    specs.decorated = false;
    specs.resizable = false;
    specs.width = 600;
    specs.height = 400;
    core.run_window(specs);
}
