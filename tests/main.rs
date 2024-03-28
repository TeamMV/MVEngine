use log::LevelFilter;
use mvcore::render::ApplicationLoopCallbacks;
use mvcore::render::window::{Window, WindowCreateInfo};

fn main() {
    mvlogger::init(std::io::stdout(), LevelFilter::Debug);

    let window = Window::new(WindowCreateInfo {
        width: 800,
        height: 600,
        title: "MVCore".to_string(),
        fullscreen: false,
        decorated: true,
        resizable: true,
        transparent: false,
        theme: None,
        vsync: false,
        fps: 60,
        ups: 30,
    });

    window.run(ApplicationLoop);
}

struct ApplicationLoop;

impl ApplicationLoopCallbacks for ApplicationLoop {
    fn start(&mut self) {

    }

    fn draw(&mut self) {

    }

    fn update(&mut self) {

    }

    fn end(&mut self) {

    }
}
