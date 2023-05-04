use mvcore::render::RenderCore;
use mvcore::render::window::WindowSpecs;

fn main() {
    let core = RenderCore::new();
    let mut specs = WindowSpecs::default();
    specs.vsync = false;
    let window = core.create_window(specs);
    window.run();
}