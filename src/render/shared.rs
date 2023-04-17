use std::cell::RefCell;
use std::rc::Rc;
use gl::types::GLuint;
use glam::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};
use glfw::ffi::{glfwGetPrimaryMonitor, glfwGetVideoMode, glfwGetWindowPos, glfwSetWindowMonitor, GLFWwindow};
use glfw::Glfw;
use mvutils::try_catch;
use mvutils::utils::TetrahedronOp;
use crate::assets::SemiAutomaticAssetManager;
use crate::render::camera::Camera;
use crate::render::draw::Draw2D;

use crate::render::opengl::{OpenGLShader, OpenGLTexture};

pub trait ApplicationLoop {
    fn start(&self, window: &mut impl Window);
    fn update(&self, window: &mut impl Window);
    fn draw(&self, window: &mut impl Window);
    fn stop(&self, window: &mut impl Window);
}

struct DefaultApplicationLoop;

impl ApplicationLoop for DefaultApplicationLoop {
    fn start(&self, window: &mut impl Window) {}

    fn update(&self, window: &mut impl Window) {}

    fn draw(&self, window: &mut impl Window) {}

    fn stop(&self, window: &mut impl Window) {}
}

pub trait Window {
    fn new(info: WindowCreateInfo, assets: Rc<RefCell<SemiAutomaticAssetManager>>) -> Self;
    fn run(&mut self, application_loop: impl ApplicationLoop);
    fn run_default(&mut self) {
        self.run(DefaultApplicationLoop {});
    }
    fn stop(&mut self);

    fn get_width(&self) -> u16;
    fn set_width(&mut self, width: u16);
    fn get_height(&self) -> u16;
    fn set_height(&mut self, height: u16);
    fn get_fps(&self) -> u16;
    fn get_ups(&self) -> u16;
    fn get_frame(&self) -> u128;

    fn get_draw_2d(&mut self) -> &mut Draw2D;
    fn set_fullscreen(&mut self, fullscreen: bool);
    fn get_glfw_window(&self) -> *mut GLFWwindow;

    fn add_shader(&mut self, id: &str, shader: Rc<RefCell<Shader>>);
    fn enable_shader(&mut self, id: &str);
    fn disable_shader(&mut self, id: &str);

    fn get_camera(&self) -> &Camera;
}

pub struct WindowCreateInfo {
    pub width: u16,
    pub height: u16,
    pub fps: u16,
    pub ups: u16,
    pub fullscreen: bool,
    pub vsync: bool,
    pub resizable: bool,
    pub decorated: bool,
    pub title: String,
}

impl WindowCreateInfo {
    pub fn new(width: u16, height: u16, fullscreen: bool, title: &str) -> Self {
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

macro_rules! backend_call {
    ($ty:ident, $s:expr, $name:ident) => {
        match $s {
            $ty::OpenGL(gl) => unsafe {
                gl.$name();
            }
        }
    };
    ($ty:ident, $s:expr, $name:ident, $($params:ident),*) => {
        match $s {
            $ty::OpenGL(gl) => unsafe {
                gl.$name($($params,)*);
            }
        }
    };
}

macro_rules! backend_ret_call {
    ($ty:ident, $s:expr, $name:ident) => {
        return match $s {
            $ty::OpenGL(gl) => unsafe {
                gl.$name()
            }
        }
    };
    ($ty:ident, $s:expr, $name:ident, $($params:ident),*) => {
        return match $s {
            $ty::OpenGL(gl) => unsafe {
                gl.$name($($params,)*)
            }
        }
    };
}

macro_rules! backend_fn {
    ($ty:ident, $name:ident) => {
        pub fn $name(&mut self) {
            backend_call!($ty, self, $name);
        }
    };
    ($ty:ident, $name:ident, $($params:ident: $types:ty),+) => {
        pub fn $name(&mut self, $($params: $types),+) {
            backend_call!($ty, self, $name $(,$params)+);
        }
    };
    ($ty:ident, $name:ident, $ret:ty) => {
        pub fn $name(&mut self) -> $ret {
            backend_ret_call!($ty, self, $name);
        }
    };
    ($ty:ident, $name:ident, $ret:ty, $($params:ident: $types:ty),+) => {
        pub fn $name(&mut self, $($params: $types),+) -> $ret {
            backend_ret_call!($ty, self, $name $(,$params)+);
        }
    };
    ($ty:ident, $name:ident, $i:expr) => {
        pub fn $name(&self) {
            backend_call!($ty, self, $name);
        }
    };
    ($ty:ident, $name:ident, $i:expr, $($params:ident: $types:ty),+) => {
        pub fn $name(&self, $($params: $types),+) {
            backend_call!($ty, self, $name $(,$params)+);
        }
    };
    ($ty:ident, $name:ident, $ret:ty, $i:expr) => {
        pub fn $name(&self) -> $ret {
            backend_ret_call!($ty, self, $name);
        }
    };
    ($ty:ident, $name:ident, $ret:ty, $i:expr, $($params:ident: $types:ty),+) => {
        pub fn $name(&self, $($params: $types),+) -> $ret {
            backend_ret_call!($ty, self, $name $(,$params)+);
        }
    };
}

pub enum Shader {
    OpenGL(OpenGLShader)
}

impl Shader {
    backend_fn!(Shader, make);
    backend_fn!(Shader, bind, true);

    backend_fn!(Shader, uniform_1f, true, name: &str, value: f32);
    backend_fn!(Shader, uniform_1i, true, name: &str, value: i32);
    pub fn uniform_1b(&self, name: &str, value: bool) {
        self.uniform_1i(name, value.yn(1, 0));
    }

    backend_fn!(Shader, uniform_fv, true, name: &str, value: &Vec<f32>);
    backend_fn!(Shader, uniform_iv, true, name: &str, value: &Vec<i32>);
    pub fn uniform_bv(&self, name: &str, value: &Vec<bool>) {
        self.uniform_iv(name, &value.iter().map(|b| {b.yn(1, 0)}).collect::<Vec<i32>>());
    }

    backend_fn!(Shader, uniform_2fv, true, name: &str, value: Vec2);
    backend_fn!(Shader, uniform_3fv, true, name: &str, value: Vec3);
    backend_fn!(Shader, uniform_4fv, true, name: &str, value: Vec4);
    backend_fn!(Shader, uniform_2fm, true, name: &str, value: Mat2);
    backend_fn!(Shader, uniform_3fm, true, name: &str, value: Mat3);
    backend_fn!(Shader, uniform_4fm, true, name: &str, value: Mat4);
}

pub enum Texture {
    OpenGL(OpenGLTexture)
}

impl Texture {
    backend_fn!(Texture, make);
    backend_fn!(Texture, bind, true, index: u8);
    backend_fn!(Texture, unbind, true);

    backend_fn!(Texture, get_id, u32, true);

    backend_fn!(Texture, get_width, u16, true);
    backend_fn!(Texture, get_height, u16, true);
}

pub struct TextureRegion {
    texture: Rc<RefCell<Texture>>,
    x: u16,
    y: u16,
    width: u16,
    height: u16
}

impl TextureRegion {
    pub(crate) fn new(texture: Rc<RefCell<Texture>>, x: u16, y: u16, width: u16, height: u16) -> Self {
        TextureRegion {
            texture,
            x,
            y,
            width,
            height
        }
    }

    pub(crate) fn from(texture: Rc<RefCell<Texture>>) -> Self {
        let width = texture.borrow().get_width();
        let height = texture.borrow().get_height();
        TextureRegion {
            texture,
            x: 0,
            y: 0,
            width,
            height
        }
    }
}

//Assets above this comment pls, here comes the "real rendering shit"

pub(crate) trait RenderProcessor2D {
    fn process_data(&self, tex: &mut [Option<Rc<RefCell<Texture>>>], tex_id: &[u32], indices: &Vec<u32>, vertices: &Vec<f32>, vbo: u32, ibo: u32, shader: &Shader, render_mode: u8);
    fn gen_buffer_id(&self) -> u32;
    fn adapt_render_mode(&self, render_mode: u8) -> u8;
}