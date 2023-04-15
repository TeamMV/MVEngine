use glfw::Glfw;
use crate::render::opengl::OpenGLWindow;
use crate::render::shared::{Window, WindowCreateInfo};

pub mod shared;
pub mod opengl;

pub struct Renderer {
    glfw: Glfw,
    backend: RenderingBackend
}

pub enum RenderingBackend {
    OpenGL
}

impl Renderer {
    pub fn new(backend: RenderingBackend) -> Self {
        let glfw = glfw::init::<String>(None).expect("Failed to initialize GLFW");
        Renderer {
            glfw,
            backend
        }
    }

    pub fn create_window(&self, info: WindowCreateInfo) -> impl Window {
        return match self.backend {
            RenderingBackend::OpenGL => {
                OpenGLWindow::new(self.glfw.clone(), info)
            }
        }
    }
}