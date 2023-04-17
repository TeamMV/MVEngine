use std::cell::RefCell;
use std::rc::Rc;
use glfw::Glfw;
use crate::assets::SemiAutomaticAssetManager;
use glfw::ffi::{CLIENT_API, DECORATED, FALSE, glfwCreateWindow, glfwDefaultWindowHints, glfwDestroyWindow, glfwGetPrimaryMonitor, glfwGetProcAddress, glfwGetVideoMode, glfwGetWindowPos, glfwPollEvents, glfwSetCharCallback, glfwSetCharModsCallback, glfwSetCursorEnterCallback, glfwSetCursorPosCallback, glfwSetDropCallback, glfwSetFramebufferSizeCallback, glfwSetKeyCallback, glfwSetMouseButtonCallback, glfwSetScrollCallback, glfwSetWindowCloseCallback, glfwSetWindowContentScaleCallback, glfwSetWindowFocusCallback, glfwSetWindowIconifyCallback, glfwSetWindowMaximizeCallback, glfwSetWindowMonitor, glfwSetWindowPosCallback, glfwSetWindowRefreshCallback, glfwSetWindowSizeCallback, glfwShowWindow, glfwSwapBuffers, glfwWindowHint, glfwWindowShouldClose, glfwInit, glfwTerminate, GLFWwindow, OPENGL_API, RESIZABLE, TRUE, VISIBLE};

use crate::render::opengl::{OpenGLShader, OpenGLTexture, OpenGLWindow};
use crate::render::shared::{Shader, Texture, Window, WindowCreateInfo};

pub mod shared;
pub mod opengl;
pub mod draw;
pub mod color;
pub mod batch;
pub mod camera;

pub unsafe fn glfwFreeCallbacks(window: *mut GLFWwindow) {
    glfwSetWindowPosCallback(window, None);
    glfwSetWindowSizeCallback(window, None);
    glfwSetWindowCloseCallback(window, None);
    glfwSetWindowRefreshCallback(window, None);
    glfwSetWindowFocusCallback(window, None);
    glfwSetWindowIconifyCallback(window, None);
    glfwSetWindowMaximizeCallback(window, None);
    glfwSetFramebufferSizeCallback(window, None);
    glfwSetWindowContentScaleCallback(window, None);
    glfwSetKeyCallback(window, None);
    glfwSetCharCallback(window, None);
    glfwSetCharModsCallback(window, None);
    glfwSetMouseButtonCallback(window, None);
    glfwSetCursorPosCallback(window, None);
    glfwSetCursorEnterCallback(window, None);
    glfwSetScrollCallback(window, None);
    glfwSetDropCallback(window, None);
}

pub struct RenderCore {
    backend: RenderingBackend,
    assets: Rc<RefCell<SemiAutomaticAssetManager>>
}

pub enum RenderingBackend {
    OpenGL
}

impl RenderCore {
    pub(crate) fn new(backend: RenderingBackend, assets: Rc<RefCell<SemiAutomaticAssetManager>>) -> Self {
        unsafe {
            glfwInit();
        }
        RenderCore {
            backend,
            assets
        }
    }

    pub(crate) fn terminate(&mut self) {
        unsafe {
            glfwTerminate();
        }
    }

    pub fn create_window(&self, info: WindowCreateInfo) -> impl Window {
        return match self.backend {
            RenderingBackend::OpenGL => {
                OpenGLWindow::new(info, self.assets.clone())
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