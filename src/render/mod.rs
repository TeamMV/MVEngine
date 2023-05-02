use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use glfw::ffi::{glfwInit, glfwSetCharCallback, glfwSetCharModsCallback, glfwSetCursorEnterCallback, glfwSetCursorPosCallback, glfwSetDropCallback, glfwSetFramebufferSizeCallback, glfwSetKeyCallback, glfwSetMouseButtonCallback, glfwSetScrollCallback, glfwSetWindowCloseCallback, glfwSetWindowContentScaleCallback, glfwSetWindowFocusCallback, glfwSetWindowIconifyCallback, glfwSetWindowMaximizeCallback, glfwSetWindowPosCallback, glfwSetWindowRefreshCallback, glfwSetWindowSizeCallback, glfwTerminate, GLFWwindow};
use is_main_thread::is_main_thread;
use crate::ApplicationInfo;

use crate::assets::{SemiAutomaticAssetManager, WritableAssetManager};
use crate::render::opengl::opengl::{OpenGLShader, OpenGLTexture, OpenGLWindow};
use crate::render::shared::{EffectShader, Shader, Texture, Window, WindowCreateInfo};

#[cfg(feature = "vulkan")]
use crate::render::vulkan::vulkan::{VulkanWindow, VulkanShader, VulkanTexture};
use crate::resource_loader::ResourceLoader;

pub mod shared;
pub mod draw;
pub mod color;
pub mod batch2d;
pub mod camera;
pub mod text;
pub mod opengl;
#[cfg(feature = "vulkan")]
pub mod vulkan;
#[cfg(feature = "3d")]
pub mod model;
#[cfg(feature = "3d")]
pub mod batch3d;
#[cfg(feature = "3d")]
pub mod lights;

pub const EFFECT_VERT: &str = "#version 450\nout vec2 fTexCoord;vec2 positions[4]=vec2[](vec2(-1.0,-1.0),vec2(-1.0,1.0),vec2(1.0,-1.0),vec2(1.0,1.0));vec2 tex[4]=vec2[](vec2(0.0,0.0),vec2(0.0,1.0),vec2(1.0,0.0),vec2(1.0,1.0));void main(){fTexCoord=tex[gl_VertexID];gl_Position=vec4(positions[gl_VertexID],0.0,1.0);}";
pub const EMPTY_EFFECT_FRAG: &str = "#version 450\nin vec2 fTexCoord;out vec4 outColor;uniform sampler2D tex;void main(){outColor=texture(tex,fTexCoord);}";

#[cfg(feature = "vulkan")]
pub const VK_EFFECT_VERT: &str = "#version 450\nlayout(location=0)out vec2 fTexCoord;vec2 positions[4]=vec2[](vec2(-1.0,-1.0),vec2(-1.0,1.0),vec2(1.0,-1.0),vec2(1.0,1.0));vec2 tex[4]=vec2[](vec2(0.0,0.0),vec2(0.0,1.0),vec2(1.0,0.0),vec2(1.0,1.0));void main(){fTexCoord=tex[gl_VertexIndex];gl_Position=vec4(positions[gl_VertexIndex],0.0,1.0);}";
#[cfg(feature = "vulkan")]
pub const VK_EMPTY_EFFECT_FRAG: &str = "#version 450\nlayout(location=0)in vec2 fTexCoord;layout(location=0)out vec4 outColor;layout(binding=0)uniform sampler2D tex;void main(){outColor=texture(tex,fTexCoord);}";

#[macro_export]
macro_rules! ensure_main_thread {
    () => {
        let is_main = is_main_thread();
        if is_main.is_none() || !is_main.unwrap() {
            return;
        }
    };
}

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

pub(crate) fn load_render_assets(assets: &mut dyn WritableAssetManager) {
    assets.load_bitmap_font("default", "fonts/font.png", "fonts/default.fnt");
    assets.load_shader("default", "shaders/default.vert", "shaders/default.frag");
    assets.load_effect_shader("blur", "effects/blur.frag");
    assets.load_effect_shader("pixelate", "effects/pixelate.frag");
    #[cfg(feature = "3d")]
    {
        assets.load_shader("model", "shaders/model_geom.vert", "shaders/model_geom.frag");
        assets.load_shader("batch", "shaders/batch_geom.vert", "shaders/batch_geom.frag");
        assets.load_effect_shader("deferred", "shaders/deferred_light.frag");
    }
}

pub struct RenderCore {
    backend: RenderingBackend,
    resource_loader: Arc<ResourceLoader>,
    app: *const ApplicationInfo
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum RenderingBackend {
    OpenGL,
    #[cfg(feature = "vulkan")]
    Vulkan
}

impl RenderCore {
    pub(crate) fn new(info: &ApplicationInfo, resource_loader: Arc<ResourceLoader>) -> Self {
        unsafe {
            glfwInit();
            RenderCore {
                backend: info.backend.clone(),
                resource_loader,
                app: info
            }
        }
    }

    pub(crate) fn terminate(&self) {
        unsafe {
            glfwTerminate();
        }
    }

    pub(crate) fn rollback(&mut self) {
        self.backend = RenderingBackend::OpenGL;
    }

    pub fn create_window(&self, info: WindowCreateInfo) -> Window {
        match self.backend {
            RenderingBackend::OpenGL => {
                Window::OpenGL(OpenGLWindow::new(info, self.resource_loader.clone()))
            }
            #[cfg(feature = "vulkan")]
            RenderingBackend::Vulkan => unsafe {
                Window::Vulkan(VulkanWindow::new(info, self.resource_loader.clone(), (self as *const _) as *mut _, self.app))
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
                    EffectShader::Vulkan(VulkanShader::new(VK_EFFECT_VERT, source))
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
                    Shader::Vulkan(VulkanShader::new(vertex, fragment))
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
                    Texture::Vulkan(VulkanTexture::new(bytes.to_vec()))
                }
            }
        }
    }
}

pub mod shader_preprocessor {
    pub(crate) const TEXTURE_LIMIT: u32 = 64;
    pub(crate) static mut MAX_TEXTURES: u32 = 0;
    pub(crate) const MAX_TEXTURES_IDENTIFIER: &str = "MAX_TEXTURE_IMAGE_UNITS";

    pub(crate) static mut MAX_NUM_LIGHTS: u32 = 0;
    pub(crate) const MAX_NUM_LIGHTS_IDENTIFIER: &str = "MAX_NUM_LIGHTS";

    pub(crate) fn process(source: &str) -> String {
        unsafe {
            source
                .replace(MAX_TEXTURES_IDENTIFIER, MAX_TEXTURES.to_string().as_str())
                .replace(MAX_NUM_LIGHTS_IDENTIFIER, MAX_NUM_LIGHTS.to_string().as_str())
        }
    }
}