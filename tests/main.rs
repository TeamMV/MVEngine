use std::sync::Arc;
use mvsync::MVSync;
use mvutils::version::Version;

use mvcore::{ApplicationInfo, MVCore};
use mvcore::render::{ApplicationLoopCallbacks, RenderCore};
use mvcore::render::window::{Window, WindowSpecs};

fn main() {
    env_logger::init();
    let core = MVCore::new(ApplicationInfo {
        name: "Test".to_string(),
        version: Version::new(1, 0, 0),
        multithreaded: true,
        extra_threads: 1,
    });
    let mut specs = WindowSpecs::default();
    specs.vsync = false;
    specs.fps = 10000;
    specs.decorated = true;
    specs.resizable = true;
    specs.width = 800;
    specs.height = 600;
    core.get_render().run_window(specs, ApplicationLoop)
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