use glfw::ClientApiHint::OpenGl;
use glfw::{Glfw, WindowMode};
use glfw::ffi::GLFWwindow;
use glfw::WindowHint::{ClientApi, Decorated, Resizable, Visible};
use glfw::WindowMode::Windowed;
use mvutils::utils::TetrahedronOp;
use crate::render::shared::{ApplicationLoop, Window, WindowCreateInfo};

pub struct OpenGLWindow {
    glfw: Glfw,
    info: WindowCreateInfo,
    window: Option<glfw::Window>
}

impl OpenGLWindow {
    fn init(&mut self, application_loop: &impl ApplicationLoop) {
        self.glfw.default_window_hints();
        self.glfw.window_hint(Visible(false));
        self.glfw.window_hint(ClientApi(OpenGl));
        self.glfw.window_hint(Resizable(self.info.resizable));
        self.glfw.window_hint(Decorated(self.info.decorated));

        let mut window = self.glfw
            .create_window(self.info.width, self.info.height, self.info.title.as_str(), Windowed)
            .expect("Failed to create window!");
        self.window = Some(window.0);

        self.get_window().show();
    }

    fn running(&mut self, application_loop: &impl ApplicationLoop) {
        while !self.get_window().should_close() {

        }
    }

    fn terminate(&mut self) {

    }

    fn get_window(&mut self) -> &mut glfw::Window {
        self.window.as_mut().unwrap()
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

    fn run(&mut self, application_loop: impl ApplicationLoop) {
        self.init(&application_loop);

        self.running(&application_loop);
        self.terminate();
    }
}