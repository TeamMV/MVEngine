use glfw::Glfw;

pub trait ApplicationLoop {
    fn start(&self, window: &mut impl Window);
}

struct DefaultApplicationLoop;

impl ApplicationLoop for DefaultApplicationLoop {
    fn start(&self, window: &mut impl Window) {}
}

pub trait Window {
    fn new(glfw: Glfw, info: WindowCreateInfo) -> Self;

    fn run(&mut self, application_loop: impl ApplicationLoop);

    fn run_default(&mut self) {
        self.run(DefaultApplicationLoop {});
    }
}

pub struct WindowCreateInfo {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub ups: u32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub resizable: bool,
    pub decorated: bool,
    pub title: String,
}

impl WindowCreateInfo {
    pub fn new(width: u32, height: u32, fullscreen: bool, title: &str) -> Self {
        WindowCreateInfo {
            width,
            height,
            fps: 60,
            ups: 20,
            fullscreen,
            vsync: false,
            resizable: true,
            decorated: true,
            title: title.to_string(),
        }
    }
}

impl Default for WindowCreateInfo {
    fn default() -> Self {
        WindowCreateInfo {
            width: 800,
            height: 600,
            fps: 60,
            ups: 20,
            fullscreen: false,
            vsync: false,
            resizable: true,
            decorated: true,
            title: String::new(),
        }
    }
}