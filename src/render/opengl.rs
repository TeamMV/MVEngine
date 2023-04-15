use std::ffi::{CStr, CString};

use gl::types::{GLint, GLuint};
use glfw::{Glfw, WindowMode};
use glfw::ClientApiHint::OpenGl;
use glfw::ffi::GLFWwindow;
use glfw::WindowHint::{ClientApi, Decorated, Resizable, Visible};
use glfw::WindowMode::Windowed;
use mvutils::utils::TetrahedronOp;

use crate::render::shared::{ApplicationLoop, Shader, Window, WindowCreateInfo};

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

pub struct OpenGLShader {
    vertex: CString,
    fragment: CString,
    prgm_id: GLuint,
    vertex_id: GLuint,
    fragment_id: GLuint
}

impl OpenGLShader {
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
}

impl Shader for OpenGLShader {
    fn new(vertex: &str, fragment: &str) -> Self {
        OpenGLShader {
            vertex: CString::new(vertex).unwrap(),
            fragment: CString::new(fragment).unwrap(),
            prgm_id: 0,
            vertex_id: 0,
            fragment_id: 0,
        }
    }

    unsafe fn make(&mut self) {
        self.prgm_id = gl::CreateProgram();
        self.vertex_id = gl::CreateShader(gl::VERTEX_SHADER);
        self.fragment_id = gl::CreateShader(gl::FRAGMENT_SHADER);
        self.create_shader(self.vertex_id, &self.vertex);
        self.create_shader(self.fragment_id, &self.fragment);
    }

    unsafe  fn bind(&mut self) {
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

    unsafe  fn take(&self) {
        gl::UseProgram(self.prgm_id)
    }
}