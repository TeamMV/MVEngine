use glfw::Glfw;

use crate::render::opengl::{OpenGLShader, OpenGLTexture, OpenGLWindow};
use crate::render::shared::{Shader, Texture, Window, WindowCreateInfo};

pub mod shared;
pub mod opengl;
pub mod draw;
pub mod color;

pub struct RenderCore {
    glfw: Glfw,
    backend: RenderingBackend,
}

pub enum RenderingBackend {
    OpenGL
}

impl RenderCore {
    pub fn new(backend: RenderingBackend) -> Self {
        let glfw = glfw::init::<String>(None).expect("Failed to initialize GLFW");
        RenderCore {
            glfw,
            backend,
        }
    }

    pub fn create_window(&self, info: WindowCreateInfo) -> impl Window {
        return match self.backend {
            RenderingBackend::OpenGL => {
                OpenGLWindow::new(self.glfw.clone(), info)
            }
        };
    }

    pub fn create_shader(&self, vertex: &str, fragment: &str) -> Shader {
        unsafe {
            return match self.backend {
                RenderingBackend::OpenGL => {
                    Shader::OpenGL(OpenGLShader::new(vertex, fragment))
                }
            };
        }
    }

    pub fn create_texture(&self, bytes: Vec<u8>) -> Texture {
        unsafe {
            return match self.backend {
                RenderingBackend::OpenGL => {
                    Texture::OpenGL(OpenGLTexture::new(bytes))
                }
            }
        }
    }
}