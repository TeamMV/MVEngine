use log::LevelFilter;
use mvengine::rendering::api::Renderer;
use mvengine::window::app::WindowCallbacks;
use mvengine::window::{Error, Window, WindowCreateInfo};
use mvutils::once::CreateOnce;
use mvutils::utils::setup_private_panic;
use mvutils::version::Version;
use parking_lot::RwLock;
use std::io::stdout;
use std::sync::Arc;

pub fn main() -> Result<(), Error> {
    mvengine::panic::setup_logger(stdout(), LevelFilter::Trace, 1000);
    mvengine::panic::setup_panic(true, "mve/logs");

    let mut info = WindowCreateInfo::default();
    info.title = "Window demo".to_string();
    info.fps = 60;
    info.ups = 20;
    info.vsync = true;

    let window = Window::new(info);
    let arc = Arc::new(RwLock::new(Application::new()));
    window.run::<Application>(arc)
}

struct Application {
    renderer: CreateOnce<Renderer>,
}

impl Application {
    fn new() -> Self {
        Self {
            renderer: CreateOnce::new(),
        }
    }
}

impl WindowCallbacks for Application {
    fn post_init(&mut self, window: &mut Window) {
        let renderer = Renderer::new_x(window, "HelloGPUApplication", Version::new(0, 1, 0, 0));
        self.renderer.create(|| renderer);
        panic!("Oh no!");
    }

    fn update(&mut self, window: &mut Window, delta_u: f64) {}

    fn draw(&mut self, window: &mut Window, delta_t: f64) {}

    fn post_draw(&mut self, window: &mut Window, delta_t: f64) {}

    fn exiting(&mut self, window: &mut Window) {}

    fn resize(&mut self, window: &mut Window, width: u32, height: u32) {}
}
