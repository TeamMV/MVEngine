use cgmath::{Matrix2, Matrix3, Matrix4, Vector2, Vector3, Vector4};
use glfw::Glfw;
use mvutils::try_catch;
use mvutils::utils::TetrahedronOp;

use crate::render::opengl::{OpenGLShader, OpenGLTexture};

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
    fn stop(&mut self);

    fn get_width(&self) -> u16;
    fn get_height(&self) -> u16;
    fn get_fps(&self) -> u16;
    fn get_ups(&self) -> u16;
    fn get_frame(&self) -> u128;
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
    backend_fn!(Shader, bind);

    backend_fn!(Shader, uniform_1f, name: &str, value: f32);
    backend_fn!(Shader, uniform_1i, name: &str, value: i32);
    pub fn uniform_1b(&mut self, name: &str, value: bool) {
        self.uniform_1i(name, value.yn(1, 0));
    }

    backend_fn!(Shader, uniform_fv, name: &str, value: &Vec<f32>);
    backend_fn!(Shader, uniform_iv, name: &str, value: &Vec<i32>);
    pub fn uniform_bv(&mut self, name: &str, value: &Vec<bool>) {
        self.uniform_iv(name, &value.iter().map(|b| {b.yn(1, 0)}).collect::<Vec<i32>>());
    }

    backend_fn!(Shader, uniform_2fv, name: &str, value: Vector2<f32>);
    backend_fn!(Shader, uniform_3fv, name: &str, value: Vector3<f32>);
    backend_fn!(Shader, uniform_4fv, name: &str, value: Vector4<f32>);
    backend_fn!(Shader, uniform_2fm, name: &str, value: Matrix2<f32>);
    backend_fn!(Shader, uniform_3fm, name: &str, value: Matrix3<f32>);
    backend_fn!(Shader, uniform_4fm, name: &str, value: Matrix4<f32>);
}

pub enum Texture {
    OpenGL(OpenGLTexture)
}

impl Texture {
    pub fn crop(&self, x: u16, y: u16, width: u16, height: u16) -> TextureRegion {
        TextureRegion {
            texture: self,
            x,
            y,
            width,
            height
        }
    }

    pub fn to_region(&self) -> TextureRegion {
        self.crop(0, 0, self.get_width(), self.get_height())
    }

    backend_fn!(Texture, make);
    backend_fn!(Texture, bind);
    backend_fn!(Texture, unbind);

    backend_fn!(Texture, get_width, u16, true);
    backend_fn!(Texture, get_height, u16, true);
}

pub struct TextureRegion {
    texture: &'static Texture,
    x: u16,
    y: u16,
    width: u16,
    height: u16
}

impl TextureRegion {

}