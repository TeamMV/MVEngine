use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;

use glfw::ffi::{glfwInit, glfwSetCharCallback, glfwSetCharModsCallback, glfwSetCursorEnterCallback, glfwSetCursorPosCallback, glfwSetDropCallback, glfwSetFramebufferSizeCallback, glfwSetKeyCallback, glfwSetMouseButtonCallback, glfwSetScrollCallback, glfwSetWindowCloseCallback, glfwSetWindowContentScaleCallback, glfwSetWindowFocusCallback, glfwSetWindowIconifyCallback, glfwSetWindowMaximizeCallback, glfwSetWindowPosCallback, glfwSetWindowRefreshCallback, glfwSetWindowSizeCallback, glfwTerminate, GLFWwindow};
use image::{EncodableLayout, ImageFormat};
use image::ImageFormat::Png;

use crate::assets::SemiAutomaticAssetManager;
use crate::render::opengl::{OpenGLShader, OpenGLTexture, OpenGLWindow};
use crate::render::shared::{EffectShader, Shader, Texture, Window, WindowCreateInfo};

pub mod shared;
pub mod opengl;
pub mod draw;
pub mod color;
pub mod batch;
pub mod camera;
pub mod text;
#[cfg(feature = "vulkan")]
pub mod vulkan;

pub const EFFECT_VERT: &str = "#version 450\nout vec2 fTexCoord;vec2 positions[4]=vec2[](vec2(-1.0,-1.0),vec2(-1.0,1.0),vec2(1.0,-1.0),vec2(1.0,1.0));vec2 tex[4]=vec2[](vec2(0.0,0.0),vec2(0.0,1.0),vec2(1.0,0.0),vec2(1.0,1.0));void main(){fTexCoord=tex[gl_VertexID];gl_Position=vec4(positions[gl_VertexID],0.0,1.0);}";
pub const EMPTY_EFFECT_FRAG: &str = "#version 450\nin vec2 fTexCoord;out vec4 outColor;uniform sampler2D tex;void main(){outColor=texture(tex,fTexCoord);}";

#[allow(non_snake_case)]
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
    assets: Rc<RefCell<SemiAutomaticAssetManager>>,
}

pub enum RenderingBackend {
    OpenGL,
    #[cfg(feature = "vulkan")]
    Vulkan
}

impl RenderCore {
    pub(crate) fn new(backend: RenderingBackend, assets: Rc<RefCell<SemiAutomaticAssetManager>>) -> Self {
        unsafe {
            glfwInit();
        }
        RenderCore {
            backend,
            assets,
        }
    }

    pub(crate) fn terminate(&self) {
        unsafe {
            glfwTerminate();
        }
    }

    pub fn create_window(&self, info: WindowCreateInfo) -> impl Window {
        match self.backend {
            RenderingBackend::OpenGL => {
                OpenGLWindow::new(info, self.assets.clone())
            }
            #[cfg(feature = "vulkan")]
            RenderingBackend::Vulkan => {
                panic!()
            }
        }
    }

    pub fn create_effect_shader(&self, source: &str) -> EffectShader {
        unsafe {
            match self.backend {
                RenderingBackend::OpenGL => {
                    EffectShader::OpenGL(OpenGLShader::new(EFFECT_VERT, source))
                }
                #[cfg(feature = "vulkan")]
                RenderingBackend::Vulkan => {
                    panic!()
                }
            }
        }
    }

    pub fn create_shader(&self, vertex: &str, fragment: &str) -> Shader {
        unsafe {
            match self.backend {
                RenderingBackend::OpenGL => {
                    Shader::OpenGL(OpenGLShader::new(vertex, fragment))
                }
                #[cfg(feature = "vulkan")]
                RenderingBackend::Vulkan => {
                    panic!()
                }
            }
        }
    }

    pub fn create_texture(&self, bytes: &[u8]) -> Texture {
        unsafe {
            match self.backend {
                RenderingBackend::OpenGL => {
                    Texture::OpenGL(OpenGLTexture::new(bytes.to_vec()))
                }
                #[cfg(feature = "vulkan")]
                RenderingBackend::Vulkan => {
                    panic!()
                }
            }
        }
    }
}