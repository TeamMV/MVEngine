use glfw::ClientApiHint::OpenGl;
use glfw::{Glfw, WindowMode};
use glfw::ffi::GLFWwindow;
use glfw::WindowHint::{ClientApi, Decorated, Resizable, Visible};
use glfw::WindowMode::Windowed;
use mvutils::utils::TetrahedronOp;
use crate::render::shared::{Window, WindowCreateInfo};

pub struct OpenGLWindow {
    glfw: Glfw,
    info: WindowCreateInfo,
    window: Option<glfw::Window>
}

impl OpenGLWindow {
    fn init<F>(&mut self, start: F) where F: FnMut(Self) {
        self.glfw.default_window_hints();
        self.glfw.window_hint(Visible(false));
        self.glfw.window_hint(ClientApi(OpenGl));
        self.glfw.window_hint(Resizable(self.info.resizable));
        self.glfw.window_hint(Decorated(self.info.decorated));

        let window = self.glfw
            .create_window(self.info.width, self.info.height, self.info.title.as_str(), Windowed)
            .expect("Failed to create window!");
        self.window = Some(window.0);
    }

    fn draw<F>(&mut self, update: F, draw: F) where F: FnMut(Self) {

    }

    fn terminate(&mut self) {

    }
}

impl Window for OpenGLWindow {
    fn new(glfw: Glfw, info: WindowCreateInfo) -> Self {
        OpenGLWindow {
            glfw,
            info,
            window: None
        }
    }

    fn run<F>(&mut self, start: F, update: F, draw: F) where F: FnMut(Self) {
        self.init(start);

        self.draw(update, draw);
        self.terminate();
    }
}