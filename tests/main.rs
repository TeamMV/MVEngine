use log::LevelFilter::Trace;
use mvcore::render::RenderCore;
use mvcore::render::window::WindowSpecs;

fn main() {
    let core = RenderCore::new();
    let mut specs = WindowSpecs::default();
    specs.vsync = false;
    specs.fps = 10000;
    core.run_window(specs);
}