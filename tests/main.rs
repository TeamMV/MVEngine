use mvcore::render::RenderCore;
use mvcore::render::window::WindowSpecs;

fn main() {
    let core = RenderCore::new();
    let specs = WindowSpecs::default();
    let window = core.create_window(specs);
    window.run();
}