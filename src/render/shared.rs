use std::cell::RefCell;
use std::rc::Rc;

use glam::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};
use mvutils::utils::{TetrahedronOp, RcMut};

use crate::render::camera::Camera;
use crate::render::draw::Draw2D;
use crate::render::opengl::opengl::{OpenGLShader, OpenGLTexture, OpenGLWindow};
#[cfg(feature = "vulkan")]
use crate::render::vulkan::vulkan::*;

pub trait ApplicationLoop {
    fn start(&mut self, window: RunningWindow);
    fn update(&mut self, window: RunningWindow);
    fn draw(&mut self, window: RunningWindow);
    fn stop(&mut self, window: RunningWindow);
}

struct DefaultApplicationLoop;

impl ApplicationLoop for DefaultApplicationLoop {
    fn start(&mut self, _:RunningWindow) {}
    fn update(&mut self, _: RunningWindow) {}
    fn draw(&mut self, _: RunningWindow) {}
    fn stop(&mut self, _: RunningWindow) {}
}

macro_rules! backend_call {
    ($ty:ident, $s:expr, $name:ident) => {
        match $s {
            #[allow(unused_unsafe)]
            $ty::OpenGL(gl) => unsafe {
                gl.$name();
            }
            #[cfg(feature = "vulkan")]
            #[allow(unused_unsafe)]
            $ty::Vulkan(vk) => unsafe {
                vk.$name();
            }
        }
    };
    ($ty:ident, $s:expr, $name:ident, $($params:ident),*) => {
        match $s {
            #[allow(unused_unsafe)]
            $ty::OpenGL(gl) => unsafe {
                gl.$name($($params,)*);
            }
            #[cfg(feature = "vulkan")]
            #[allow(unused_unsafe)]
            $ty::Vulkan(vk) => unsafe {
                vk.$name($($params,)*);
            }
        }
    };
}

macro_rules! backend_ret_call {
    ($ty:ident, $s:expr, $name:ident) => {
        return match $s {
            #[allow(unused_unsafe)]
            $ty::OpenGL(gl) => unsafe {
                gl.$name()
            }
            #[cfg(feature = "vulkan")]
            #[allow(unused_unsafe)]
            $ty::Vulkan(vk) => unsafe {
                vk.$name()
            }
        }
    };
    ($ty:ident, $s:expr, $name:ident, $($params:ident),*) => {
        return match $s {
            #[allow(unused_unsafe)]
            $ty::OpenGL(gl) => unsafe {
                gl.$name($($params,)*)
            }
            #[cfg(feature = "vulkan")]
            #[allow(unused_unsafe)]
            $ty::Vulkan(vk) => unsafe {
                vk.$name($($params,)*)
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

macro_rules! backend_ptr_call {
    ($ty:ident, $s:expr, $name:ident) => {
        match $s {
            $ty::OpenGL(gl) => unsafe {
                gl.as_mut().unwrap().$name();
            }
            #[cfg(feature = "vulkan")]
            $ty::Vulkan(vk) => unsafe {
                vk.as_mut().unwrap().$name();
            }
        }
    };
    ($ty:ident, $s:expr, $name:ident, $($params:ident),*) => {
        match $s {
            $ty::OpenGL(gl) => unsafe {
                gl.as_mut().unwrap().$name($($params,)*);
            }
            #[cfg(feature = "vulkan")]
            $ty::Vulkan(vk) => unsafe {
                vk.as_mut().unwrap().$name($($params,)*);
            }
        }
    };
}

macro_rules! backend_ptr_ret_call {
    ($ty:ident, $s:expr, $name:ident) => {
        return match $s {
            #[allow(unused_unsafe)]
            $ty::OpenGL(gl) => unsafe {
                gl.as_mut().unwrap().$name()
            }
            #[cfg(feature = "vulkan")]
            #[allow(unused_unsafe)]
            $ty::Vulkan(vk) => unsafe {
                vk.as_mut().unwrap().$name()
            }
        }
    };
    ($ty:ident, $s:expr, $name:ident, $($params:ident),*) => {
        return match $s {
            $ty::OpenGL(gl) => unsafe {
                gl.as_mut().unwrap().$name($($params,)*)
            }
            #[cfg(feature = "vulkan")]
            $ty::Vulkan(vk) => unsafe {
                vk.as_mut().unwrap().$name($($params,)*)
            }
        }
    };
}

macro_rules! backend_ptr_fn {
    ($ty:ident, $name:ident) => {
        pub fn $name(&mut self) {
            backend_ptr_call!($ty, self, $name);
        }
    };
    ($ty:ident, $name:ident, $($params:ident: $types:ty),+) => {
        pub fn $name(&mut self, $($params: $types),+) {
            backend_ptr_call!($ty, self, $name $(,$params)+);
        }
    };
    ($ty:ident, $name:ident, $ret:ty) => {
        pub fn $name(&mut self) -> $ret {
            backend_ptr_ret_call!($ty, self, $name);
        }
    };
    ($ty:ident, $name:ident, $ret:ty, $($params:ident: $types:ty),+) => {
        pub fn $name(&mut self, $($params: $types),+) -> $ret {
            backend_ptr_ret_call!($ty, self, $name $(,$params)+);
        }
    };
    ($ty:ident, $name:ident, $i:expr) => {
        pub fn $name(&self) {
            backend_ptr_call!($ty, self, $name);
        }
    };
    ($ty:ident, $name:ident, $i:expr, $($params:ident: $types:ty),+) => {
        pub fn $name(&self, $($params: $types),+) {
            backend_ptr_call!($ty, self, $name $(,$params)+);
        }
    };
    ($ty:ident, $name:ident, $ret:ty, $i:expr) => {
        pub fn $name(&self) -> $ret {
            backend_ptr_ret_call!($ty, self, $name);
        }
    };
    ($ty:ident, $name:ident, $ret:ty, $i:expr, $($params:ident: $types:ty),+) => {
        pub fn $name(&self, $($params: $types),+) -> $ret {
            backend_ptr_ret_call!($ty, self, $name $(,$params)+);
        }
    };
}

pub enum Window {
    OpenGL(OpenGLWindow),
    #[cfg(feature = "vulkan")]
    Vulkan(VulkanWindow),
}

impl Window {
    backend_fn!(Window, run, application_loop: impl ApplicationLoop);

    pub fn run_default(&mut self) {
        self.run(DefaultApplicationLoop {});
    }

    backend_fn!(Window, add_shader, id: &str, shader: Rc<RefCell<EffectShader>>);
}

pub enum RunningWindow {
    OpenGL(*mut OpenGLWindow),
    #[cfg(feature = "vulkan")]
    Vulkan(*mut VulkanWindow),
}

impl RunningWindow {
    backend_ptr_fn!(RunningWindow, stop);

    backend_ptr_fn!(RunningWindow, get_width, i32, true);
    backend_ptr_fn!(RunningWindow, get_height, i32, true);
    backend_ptr_fn!(RunningWindow, get_dpi, f32, true);
    backend_ptr_fn!(RunningWindow, get_fps, u16, true);
    backend_ptr_fn!(RunningWindow, get_ups, u16, true);
    backend_ptr_fn!(RunningWindow, get_frame, u64, true);

    backend_ptr_fn!(RunningWindow, get_draw_2d, RcMut<Draw2D>);
    backend_ptr_fn!(RunningWindow, set_fullscreen, fullscreen : bool);
    //backend_fn!(Window, get_glfw_window, *mut GLFWwindow, true);

    backend_ptr_fn!(RunningWindow, add_shader, id: &str, shader: Rc<RefCell<EffectShader>>);
    backend_ptr_fn!(RunningWindow, queue_shader_pass, info: ShaderPassInfo);

    backend_ptr_fn!(RunningWindow, get_camera, &Camera, true);
}

pub struct ShaderPassInfo {
    id: String,
    applier: Box<dyn Fn(&mut EffectShader)>,
}

impl ShaderPassInfo {
    pub fn new(id: &str, applier: impl Fn(&mut EffectShader) + 'static) -> Self {
        ShaderPassInfo {
            id: id.to_string(),
            applier: Box::new(applier),
        }
    }

    pub fn id(id: &str) -> Self {
        Self::new(id, |_| {})
    }

    pub(crate) fn apply(&self, shader: &mut EffectShader) {
        (self.applier)(shader);
    }

    pub(crate) fn get_id(&self) -> &str {
        &self.id
    }
}

#[derive(Clone)]
pub struct WindowCreateInfo {
    pub width: i32,
    pub height: i32,
    pub fps: u16,
    pub ups: u16,
    pub fullscreen: bool,
    pub vsync: bool,
    pub resizable: bool,
    pub decorated: bool,
    pub title: String,
}

impl WindowCreateInfo {
    pub fn new(width: i32, height: i32, fullscreen: bool, title: &str) -> Self {
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

pub enum Shader {
    OpenGL(OpenGLShader),
    #[cfg(feature = "vulkan")]
    Vulkan(VulkanShader)
}

impl Shader {
    backend_fn!(Shader, bind);

    backend_fn!(Shader, uniform_1f, name: &str, value: f32);
    backend_fn!(Shader, uniform_1i, name: &str, value: i32);
    pub fn uniform_1b(&mut self, name: &str, value: bool) {
        self.uniform_1i(name, value.yn(1, 0));
    }

    backend_fn!(Shader, uniform_fv, name: &str, value: &[f32]);
    backend_fn!(Shader, uniform_iv, name: &str, value: &[i32]);

    pub fn uniform_bv(&mut self, name: &str, value: &[bool]) {
        self.uniform_iv(name, value.iter().map(|b| { b.yn(1, 0) }).collect::<Vec<i32>>().as_slice());
    }

    backend_fn!(Shader, uniform_2fv, name: &str, value: Vec2);
    backend_fn!(Shader, uniform_3fv, name: &str, value: Vec3);
    backend_fn!(Shader, uniform_4fv, name: &str, value: Vec4);
    backend_fn!(Shader, uniform_2fm, name: &str, value: Mat2);
    backend_fn!(Shader, uniform_3fm, name: &str, value: Mat3);
    backend_fn!(Shader, uniform_4fm, name: &str, value: Mat4);

    #[cfg(feature = "vulkan")]
    pub(crate) fn get_vk(&mut self) -> &mut VulkanShader {
        match self {
            Shader::Vulkan(shader) => shader,
            _ => panic!(),
        }
    }
}

pub enum EffectShader {
    OpenGL(OpenGLShader),
    #[cfg(feature = "vulkan")]
    Vulkan(VulkanShader)
}

impl EffectShader {
    backend_fn!(EffectShader, bind);

    backend_fn!(EffectShader, uniform_1f, name: &str, value: f32);
    backend_fn!(EffectShader, uniform_1i, name: &str, value: i32);
    pub fn uniform_1b(&mut self, name: &str, value: bool) {
        self.uniform_1i(name, value.yn(1, 0));
    }

    backend_fn!(EffectShader, uniform_fv, name: &str, value: &[f32]);
    backend_fn!(EffectShader, uniform_iv, name: &str, value: &[i32]);
    pub fn uniform_bv(&mut self, name: &str, value: &[bool]) {
        self.uniform_iv(name, value.iter().map(|b| { b.yn(1, 0) }).collect::<Vec<i32>>().as_slice());
    }

    backend_fn!(EffectShader, uniform_2fv, name: &str, value: Vec2);
    backend_fn!(EffectShader, uniform_3fv, name: &str, value: Vec3);
    backend_fn!(EffectShader, uniform_4fv, name: &str, value: Vec4);
    backend_fn!(EffectShader, uniform_2fm, name: &str, value: Mat2);
    backend_fn!(EffectShader, uniform_3fm, name: &str, value: Mat3);
    backend_fn!(EffectShader, uniform_4fm, name: &str, value: Mat4);

    #[cfg(feature = "vulkan")]
    pub(crate) fn get_vk(&mut self) -> &mut VulkanShader {
        match self {
            EffectShader::Vulkan(shader) => shader,
            _ => panic!(),
        }
    }
}

pub enum Texture {
    OpenGL(OpenGLTexture),
    #[cfg(feature = "vulkan")]
    Vulkan(VulkanTexture)
}

impl Texture {
    backend_fn!(Texture, bind, index: u8);
    backend_fn!(Texture, make);
    backend_fn!(Texture, unbind);

    backend_fn!(Texture, get_id, u32, true);

    backend_fn!(Texture, get_width, u32, true);
    backend_fn!(Texture, get_height, u32, true);
}

pub struct TextureRegion {
    texture: Rc<RefCell<Texture>>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    uv: [f32; 4]
}

impl TextureRegion {
    pub fn new(texture: Rc<RefCell<Texture>>, x: u32, y: u32, width: u32, height: u32) -> Self {
        let w = texture.borrow().get_width() as f32;
        let h = texture.borrow().get_height() as f32;
        TextureRegion {
            texture,
            x,
            y,
            width,
            height,
            uv: [x as f32 / w, (x + width) as f32 / w, y as f32 / h, (y + height) as f32 / h]
        }
    }

    pub fn from(texture: Rc<RefCell<Texture>>) -> Self {
        let width = texture.borrow().get_width();
        let height = texture.borrow().get_height();
        TextureRegion {
            texture,
            x: 0,
            y: 0,
            width,
            height,
            uv: [0.0, 0.0, 1.0, 1.0],
        }
    }

    pub(crate) fn get_uv(&self) -> [f32; 4] {
        self.uv
    }

    pub(crate) fn parent(&self) -> Rc<RefCell<Texture>> {
        self.texture.clone()
    }
}

//Assets above this comment pls, here comes the "real rendering shit"

pub(crate) trait RenderProcessor2D {
    #[allow(clippy::too_many_arguments)]
    fn process_data(&self, tex: &mut [Option<Rc<RefCell<Texture>>>], tex_id: &[u32], indices: &[u32], vertices: &[f32], vbo: u32, ibo: u32, shader: &mut Shader, render_mode: u8);
    fn gen_buffer_id(&self) -> u32;
    fn adapt_render_mode(&self, render_mode: u8) -> u8;
}