use std::ffi::{c_int, c_void, CStr, CString};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::borrow::Borrow;
use std::cell::RefCell;

use cgmath::{Matrix2, Matrix3, Matrix4, Vector2, Vector3, Vector4, Array, Matrix, Zero};
use gl::{COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT, FLOAT};
use gl::types::{GLdouble, GLenum, GLint, GLsizei, GLsizeiptr, GLuint};
use glfw::{Context, Glfw, WindowMode};
use glfw::ClientApiHint::OpenGl;
use glfw::ffi::{CLIENT_API, DECORATED, FALSE, glfwCreateWindow, glfwDefaultWindowHints, glfwDestroyWindow, glfwGetPrimaryMonitor, glfwGetProcAddress, glfwGetVideoMode, glfwGetWindowPos, glfwInit, glfwMakeContextCurrent, glfwPollEvents, glfwSetCharCallback, glfwSetCharModsCallback, glfwSetCursorEnterCallback, glfwSetCursorPosCallback, glfwSetDropCallback, glfwSetFramebufferSizeCallback, glfwSetKeyCallback, glfwSetMouseButtonCallback, glfwSetScrollCallback, glfwSetWindowCloseCallback, glfwSetWindowContentScaleCallback, glfwSetWindowFocusCallback, glfwSetWindowIconifyCallback, glfwSetWindowMaximizeCallback, glfwSetWindowMonitor, glfwSetWindowPosCallback, glfwSetWindowRefreshCallback, glfwSetWindowShouldClose, glfwSetWindowSizeCallback, glfwShowWindow, glfwSwapBuffers, glfwSwapInterval, GLFWwindow, glfwWindowHint, glfwWindowShouldClose, OPENGL_API, RESIZABLE, TRUE, VISIBLE};
use glfw::WindowHint::{ClientApi, Decorated, Resizable, Visible};
use glfw::WindowMode::Windowed;
use mvutils::utils::{AsCStr, IncDec, TetrahedronOp, Time};

use crate::assets;
use crate::assets::SemiAutomaticAssetManager;
use crate::render::batch::batch_layout_2d;
use crate::render::batch::batch_layout_2d::{POSITION_OFFSET_BYTES, POSITION_SIZE, VERTEX_SIZE_BYTES};
use crate::render::draw::Draw2D;
use crate::render::glfwFreeCallbacks;
use crate::render::shared::{ApplicationLoop, RenderProcessor2D, Shader, Texture, Window, WindowCreateInfo};

pub struct OpenGLWindow {
    info: WindowCreateInfo,
    assets: Rc<RefCell<SemiAutomaticAssetManager>>,
    window: *mut GLFWwindow,
    current_fps: u16,
    current_ups: u16,
    current_frame: u128,

    size_buf: Vector4<i32>,

    draw_2d: Option<Draw2D<OpenGLWindow>>,

    z_near: f32,
    z_far: f32
}

impl OpenGLWindow {
    fn init(&mut self) {
        unsafe {
            glfwDefaultWindowHints();
            glfwWindowHint(VISIBLE, FALSE);
            glfwWindowHint(CLIENT_API, OPENGL_API);
            glfwWindowHint(DECORATED, self.info.decorated.yn(TRUE, FALSE));
            glfwWindowHint(RESIZABLE, self.info.resizable.yn(TRUE, FALSE));

            self.window = glfwCreateWindow(self.info.width as i32, self.info.height as i32, self.info.title.as_c_str().as_ptr(), 0 as *mut _, 0 as *mut _);

            glfwMakeContextCurrent(self.window);
            glfwSwapInterval(self.info.vsync.yn(1, 0));

            gl::load_with(|s| glfwGetProcAddress(s.as_c_str().as_ptr()));

            glfwShowWindow(self.window);

            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_CONSTANT_ALPHA);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthMask(gl::TRUE);
            gl::DepthFunc(gl::LEQUAL);
            gl::DepthRange(self.z_near as GLdouble, self.z_far as GLdouble);

        }
    }

    fn running(&mut self, application_loop: &impl ApplicationLoop) {
        unsafe {
            let mut init_time: u128 = u128::time_nanos();
            let mut current_time = init_time;
            let time_u = 1000000000.0 / self.info.ups as f32;
            let time_f = 1000000000.0 / self.info.fps as f32;
            let mut delta_u: f32 = 0.0;
            let mut delta_f: f32 = 0.0;
            let mut frames = 0;
            let mut ticks = 0;
            let mut timer = u128::time_millis();
            while glfwWindowShouldClose(self.window) == FALSE {
                current_time = u128::time_nanos();
                delta_u += (current_time - init_time) as f32 / time_u;
                delta_f += (current_time - init_time) as f32 / time_f;
                init_time = current_time;
                glfwPollEvents();
                if delta_u >= 1.0 {
                    //updates

                    application_loop.update(self);
                    ticks += 1;
                    delta_u -= 1.0;
                }
                if delta_f >= 1.0 {
                    unsafe {
                        gl::Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
                    }
                    //draws

                    application_loop.draw(self);
                    glfwSwapBuffers(self.window);
                    self.current_frame += 1;
                    frames += 1;
                    delta_f -= 1.0;
                }
                if u128::time_millis() - timer > 1000 {
                    self.current_ups = ticks;
                    self.current_fps = frames;
                    frames = 0;
                    ticks = 0;
                    timer += 1000;
                    println!("{}", self.current_fps);
                }
            }
        }
    }

    fn terminate(&mut self) {
        unsafe {
            glfwFreeCallbacks(self.window);
            glfwDestroyWindow(self.window);
        }
    }
}

impl Window for OpenGLWindow {
    fn new(info: WindowCreateInfo, assets: Rc<RefCell<SemiAutomaticAssetManager>>) -> Self {
        OpenGLWindow {
            info,
            assets,
            window: 0 as *mut _,
            current_fps: 0,
            current_ups: 0,
            current_frame: 0,

            size_buf: Vector4::zero(),

            draw_2d: None,
            z_near: 0.01,
            z_far: 2000.0,
        }
    }

    fn run(&mut self, application_loop: impl ApplicationLoop) {
        self.init();

        if self.info.fullscreen {
            self.set_fullscreen(true);
        }

        application_loop.start(self);

        self.running(&application_loop);
        application_loop.stop(self);
        self.terminate();
    }

    fn stop(&mut self) {
        unsafe {
            glfwSetWindowShouldClose(self.window, TRUE);
        }
    }

    fn get_width(&self) -> u16 {
        self.info.width
    }

    fn set_width(&mut self, width: u16) {
        self.info.width = width;
    }

    fn get_height(&self) -> u16 {
        self.info.height
    }

    fn set_height(&mut self, height: u16) {
        self.info.height = height;
    }

    fn get_fps(&self) -> u16 {
        self.current_fps
    }

    fn get_ups(&self) -> u16 {
        self.current_ups
    }

    fn get_frame(&self) -> u128 {
        self.current_frame
    }

    fn get_draw_2d(&self) -> &Draw2D<Self> {
        self.draw_2d.as_ref().expect("The Draw2D is not initialized yet!")
    }

    fn get_glfw_window(&self) -> *mut GLFWwindow {
        self.window
    }

    fn set_fullscreen(&mut self, fullscreen: bool) {
        unsafe {
            self.info.fullscreen = fullscreen;
            if fullscreen {
                glfwGetWindowPos(self.window, &mut self.size_buf.x as *mut _, &mut self.size_buf.y as *mut _);
                self.size_buf.z = self.info.width as i32;
                self.size_buf.w = self.info.height as i32;
                let monitor = glfwGetPrimaryMonitor();
                let mode = glfwGetVideoMode(monitor);
                glfwSetWindowMonitor(self.window, monitor, 0, 0, (*mode).width, (*mode).height, (*mode).refreshRate);
                self.info.width = (*mode).width as u16;
                self.info.height = (*mode).height as u16;
            } else {
                let monitor = glfwGetPrimaryMonitor();
                let mode = glfwGetVideoMode(monitor);
                glfwSetWindowMonitor(self.window, 0 as *mut _, self.size_buf.x, self.size_buf.y, self.size_buf.z, self.size_buf.w, (*mode).refreshRate);
            }
        }
    }
}

pub struct OpenGLShader {
    vertex: CString,
    fragment: CString,
    prgm_id: GLuint,
    vertex_id: GLuint,
    fragment_id: GLuint,
}

macro_rules! shader_uniform {
    ($uni:ident, $id:expr, $name:expr, $($params:expr),*) => {
        let loc: GLint = gl::GetUniformLocation($id, $name.as_c_str().as_ptr());
        if loc != -1 {
            gl::$uni(loc, $($params,)*);
        }
    };
}

impl OpenGLShader {
    pub(crate) unsafe fn new(vertex: &str, fragment: &str) -> Self {
        OpenGLShader {
            vertex: CString::new(vertex).unwrap(),
            fragment: CString::new(fragment).unwrap(),
            prgm_id: 0,
            vertex_id: 0,
            fragment_id: 0,
        }
    }

    unsafe fn create_shader(&self, id: GLuint, src: &CString) {
        gl::ShaderSource(id, 1, &src.as_ptr(), std::ptr::null());
        gl::CompileShader(id);

        let mut success: GLint = 0;
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        if success != 1 {
            let mut error_log_size: GLint = 0;
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut error_log_size);
            let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);
            gl::GetShaderInfoLog(id, error_log_size, &mut error_log_size, error_log.as_mut_ptr() as *mut _);

            error_log.set_len(error_log_size as usize);
            let log = String::from_utf8(error_log).unwrap();
            panic!("{}", log);
        }
    }

    pub(crate) unsafe fn make(&mut self) {
        self.prgm_id = gl::CreateProgram();
        self.vertex_id = gl::CreateShader(gl::VERTEX_SHADER);
        self.fragment_id = gl::CreateShader(gl::FRAGMENT_SHADER);
        self.create_shader(self.vertex_id, &self.vertex);
        self.create_shader(self.fragment_id, &self.fragment);

        gl::AttachShader(self.prgm_id, self.vertex_id);
        gl::AttachShader(self.prgm_id, self.fragment_id);

        gl::LinkProgram(self.prgm_id);
        let mut success: GLint = 0;
        gl::GetProgramiv(self.prgm_id, gl::LINK_STATUS, &mut success);

        if success != 1 {
            let mut error_log_size: GLint = 0;
            gl::GetProgramiv(self.prgm_id, gl::INFO_LOG_LENGTH, &mut error_log_size);
            let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);
            gl::GetProgramInfoLog(self.prgm_id, error_log_size, &mut error_log_size, error_log.as_mut_ptr() as *mut _);

            error_log.set_len(error_log_size as usize);
            let log = String::from_utf8(error_log).unwrap();
            panic!("{}", log);
        }
    }

    pub(crate) unsafe fn bind(&self) {
        gl::UseProgram(self.prgm_id)
    }

    pub(crate) unsafe fn uniform_1f(&self, name: &str, value: f32) {
        shader_uniform!(Uniform1f, self.prgm_id, name, value);
    }

    pub(crate) unsafe fn uniform_1i(&self, name: &str, value: i32) {
        shader_uniform!(Uniform1i, self.prgm_id, name, value);
    }

    pub(crate) unsafe fn uniform_fv(&self, name: &str, value: &Vec<f32>) {
        shader_uniform!(Uniform1fv, self.prgm_id, name, value.len() as i32, value.as_ptr());
    }

    pub(crate) unsafe fn uniform_iv(&self, name: &str, value: &Vec<i32>) {
        shader_uniform!(Uniform1iv, self.prgm_id, name, value.len() as i32, value.as_ptr());
    }

    pub(crate) unsafe fn uniform_2fv(&self, name: &str, value: Vector2<f32>) {
        shader_uniform!(Uniform2fv, self.prgm_id, name, 2, value.as_ptr());
    }

    pub(crate) unsafe fn uniform_3fv(&self, name: &str, value: Vector3<f32>) {
        shader_uniform!(Uniform3fv, self.prgm_id, name, 2, value.as_ptr());
    }

    pub(crate) unsafe fn uniform_4fv(&self, name: &str, value: Vector4<f32>) {
        shader_uniform!(Uniform4fv, self.prgm_id, name, 2, value.as_ptr());
    }

    pub(crate) unsafe fn uniform_2fm(&self, name: &str, value: Matrix2<f32>) {
        shader_uniform!(UniformMatrix2fv, self.prgm_id, name, 4, 0, value.as_ptr());
    }

    pub(crate) unsafe fn uniform_3fm(&self, name: &str, value: Matrix3<f32>) {
        shader_uniform!(UniformMatrix3fv, self.prgm_id, name, 9, 0, value.as_ptr());
    }

    pub(crate) unsafe fn uniform_4fm(&self, name: &str, value: Matrix4<f32>) {
        shader_uniform!(UniformMatrix4fv, self.prgm_id, name, 16, 0, value.as_ptr());
    }
}

pub struct OpenGLTexture {
    bytes: Vec<u8>,
    width: u16,
    height: u16,
    gl_id: u32
}

impl OpenGLTexture {
    pub(crate) unsafe fn new(bytes: Vec<u8>) -> Self {
        OpenGLTexture {
            bytes,
            width: 0,
            height: 0,
            gl_id: 0
        }
    }

    pub(crate) unsafe fn make(&mut self) {
        gl::GenTextures(1, &mut self.gl_id);
        gl::BindTexture(gl::TEXTURE_2D, self.gl_id);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, self.width as GLsizei, self.height as GLsizei, 0, gl::RGBA, gl::UNSIGNED_BYTE, self.bytes.as_ptr() as *const c_void);
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }

    pub(crate) unsafe fn bind(&self, index: u8) {
        gl::ActiveTexture(gl::TEXTURE0 + index as u32);
        gl::BindTexture(gl::TEXTURE_2D, self.gl_id);
    }

    pub(crate) unsafe fn unbind(&self) {
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }

    pub(crate) fn get_width(&self) -> u16 {
        self.width
    }

    pub(crate) fn get_height(&self) -> u16 {
        self.height
    }

    pub(crate) fn get_id(&self) -> u32 {
        self.gl_id
    }
}

//"real rendering" coming here

pub(crate) struct OpenGLRenderProcessor2D<Win: Window> {
    window: Win
}

macro_rules! vert_attrib {
    ($idx:expr, $size:ident, $off:ident) => {
        gl::VertexAttribPointer($idx, batch_layout_2d::$size as GLint, gl::FLOAT, 0, batch_layout_2d::VERTEX_SIZE_BYTES as GLsizei, batch_layout_2d::$off as *const _);
    };
}

impl<Win: Window> RenderProcessor2D<Win> for OpenGLRenderProcessor2D<Win> {
    fn process_data(&self, tex: &mut [Option<Rc<RefCell<Texture>>>], tex_id: &[u32], indices: &Vec<u32>, vertices: &Vec<f32>, vbo: u32, ibo: u32, shader: &Shader, render_mode: u8) {
        let mut i: u8 = 0;
        for op in tex.iter_mut() {
            if let Some(t) = op {
                t.borrow_mut().bind(i);
                i += 1;
            }
        }

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, vertices.len() as GLsizeiptr, vertices.as_ptr() as *const c_void, gl::DYNAMIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, indices.len() as GLsizeiptr, indices.as_ptr() as *const c_void, gl::DYNAMIC_DRAW);

            if !tex.is_empty() {
                shader.uniform_iv("TEX_SAMPLER", &tex_id.clone().into_iter().map(|u| {u.clone() as i32}).collect::<Vec<_>>());
            }

            shader.uniform_1i("uResX", self.window.get_width() as i32);
            shader.uniform_1i("uResY", self.window.get_width() as i32);
            //TODO: Matrices over here

            //gl::VertexAttribPointer(0, POSITION_SIZE as GLint, FLOAT, 0, VERTEX_SIZE_BYTES as GLsizei, POSITION_OFFSET_BYTES as *const _);

            vert_attrib!(0, POSITION_SIZE, POSITION_OFFSET_BYTES);
            vert_attrib!(1, ROTATION_SIZE, ROTATION_OFFSET_BYTES);
            vert_attrib!(2, ROTATION_ORIGIN_SIZE, ROTATION_ORIGIN_OFFSET_BYTES);
            vert_attrib!(3, COLOR_SIZE, COLOR_OFFSET_BYTES);
            vert_attrib!(4, UV_SIZE, UV_OFFSET_BYTES);
            vert_attrib!(5, TEX_ID_SIZE, TEX_ID_OFFSET);
            vert_attrib!(6, CANVAS_COORDS_SIZE, CANVAS_COORDS_OFFSET);
            vert_attrib!(7, CANVAS_DATA_SIZE, CANVAS_DATA_OFFSET);
            vert_attrib!(8, USE_CAMERA_SIZE, USE_CAMERA_OFFSET);

            gl::DrawElements(render_mode as GLenum, indices.len() as GLsizei, gl::UNSIGNED_INT, 0 as *const _);
        }
    }

    fn gen_buffer_id(&self) -> u32 {
        unsafe {
            let mut buf: GLuint = 0;
            gl::GenBuffers(1, &mut buf);
            buf
        }
    }

    fn adapt_render_mode(&self, render_mode: u8) -> u8 {
        render_mode
    }
}