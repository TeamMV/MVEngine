pub mod default;
pub mod light;

use crate::math::mat::{Mat2, Mat3, Mat4};
use crate::math::vec::{Vec2, Vec3, Vec4};
use gl::types::{GLint, GLsizei, GLuint};
use std::ffi::CString;
use std::ptr;
use std::str::FromStr;

#[derive(Clone)]
pub struct OpenGLShader {
    vertex_code: String,
    fragment_code: String,
    vertex_shader: GLuint,
    fragment_shader: GLuint,
    program_id: GLuint,
}

impl OpenGLShader {
    pub fn new(vertex_code: &'static str, fragment_code: &'static str) -> Self {
        OpenGLShader {
            vertex_code: vertex_code.to_string(),
            fragment_code: fragment_code.to_string(),
            vertex_shader: 0,
            fragment_shader: 0,
            program_id: 0,
        }
    }

    pub fn make(&mut self) -> Result<(), String> {
        unsafe {
            self.program_id = gl::CreateProgram();

            self.vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let vertex_code_cstr = CString::new(self.vertex_code.as_str()).unwrap();
            let vertex_code_ptr = vertex_code_cstr.as_ptr();
            let double_ptr = [vertex_code_ptr].as_ptr();
            gl::ShaderSource(self.vertex_shader, 1, double_ptr, ptr::null());
            gl::CompileShader(self.vertex_shader);
            if Self::check_shader_compile_status(self.vertex_shader).is_err() {
                return Err(format!(
                    "Vertex shader compilation error: {}",
                    Self::get_shader_log(self.vertex_shader)
                ));
            }

            self.fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let fragment_code_cstr = CString::new(self.fragment_code.as_str()).unwrap();
            let fragment_code_ptr = fragment_code_cstr.as_ptr();
            let double_ptr = [fragment_code_ptr].as_ptr();
            gl::ShaderSource(self.fragment_shader, 1, double_ptr, ptr::null());
            gl::CompileShader(self.fragment_shader);
            if Self::check_shader_compile_status(self.fragment_shader).is_err() {
                return Err(format!(
                    "Fragment shader compilation error: {}",
                    Self::get_shader_log(self.fragment_shader)
                ));
            }

            Ok(())
        }
    }

    fn compile_shader(shader: GLuint, source_code: &str) -> Result<(), String> {
        unsafe {
            let lines: Vec<CString> = source_code
                .lines()
                .map(|line| CString::new(line).unwrap())
                .collect();

            let line_ptrs: Vec<*const i8> = lines.iter().map(|line| line.as_ptr()).collect();
            let line_lengths: Vec<GLint> = lines.iter().map(|line| line.to_bytes().len() as GLint).collect();

            gl::ShaderSource(shader, line_ptrs.len() as GLsizei, line_ptrs.as_ptr(), line_lengths.as_ptr());
            gl::CompileShader(shader);

            if Self::check_shader_compile_status(shader).is_err() {
                return Err(Self::get_shader_log(shader));
            }
        }

        Ok(())
    }

    pub fn bind(&self) -> Result<(), String> {
        unsafe {
            gl::AttachShader(self.program_id, self.vertex_shader);
            gl::AttachShader(self.program_id, self.fragment_shader);
            gl::LinkProgram(self.program_id);

            if Self::check_program_link_status(self.program_id).is_err() {
                return Err(format!(
                    "Program link error: {}",
                    Self::get_program_log(self.program_id)
                ));
            }

            gl::ValidateProgram(self.program_id);
            if Self::check_program_validate_status(self.program_id).is_err() {
                return Err(format!(
                    "Program validate error: {}",
                    Self::get_program_log(self.program_id)
                ));
            }

            Ok(())
        }
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.program_id);
        }
    }

    pub fn get_program_id(&self) -> GLuint {
        self.program_id
    }

    fn check_shader_compile_status(shader: GLuint) -> Result<(), String> {
        unsafe {
            let mut success = 0;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                return Err("Shader compilation failed.".to_string());
            }
        }
        Ok(())
    }

    fn get_shader_log(shader: GLuint) -> String {
        unsafe {
            let mut log_length = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_length);

            let mut log = Vec::with_capacity(log_length as usize);
            gl::GetShaderInfoLog(shader, log_length, &mut log_length, log.as_mut_ptr() as *mut i8);

            log.set_len(log_length as usize);
            String::from_utf8_lossy(&log).to_string()
        }
    }

    fn check_program_link_status(program: GLuint) -> Result<(), String> {
        unsafe {
            let mut success = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                return Err("Program linking failed.".to_string());
            }
        }
        Ok(())
    }

    fn get_program_log(program: GLuint) -> String {
        unsafe {
            let mut log_length = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_length);

            let mut log = Vec::with_capacity(log_length as usize);
            gl::GetProgramInfoLog(program, log_length, &mut log_length, log.as_mut_ptr() as *mut i8);

            log.set_len(log_length as usize);
            String::from_utf8_lossy(&log).to_string()
        }
    }

    fn check_program_validate_status(program: GLuint) -> Result<(), String> {
        unsafe {
            let mut success = 0;
            gl::GetProgramiv(program, gl::VALIDATE_STATUS, &mut success);
            if success == 0 {
                return Err("Program validation failed.".to_string());
            }
        }
        Ok(())
    }

    pub fn uniform_1f(&self, name: &str, value: f32) {
        unsafe {
            let location = gl::GetUniformLocation(self.program_id, CString::new(name).unwrap().as_ptr());
            if location != -1 {
                gl::Uniform1f(location, value);
            }
        }
    }

    pub fn uniform_1i(&self, name: &str, value: i32) {
        unsafe {
            let location = gl::GetUniformLocation(self.program_id, CString::new(name).unwrap().as_ptr());
            if location != -1 {
                gl::Uniform1i(location, value);
            }
        }
    }

    pub(crate) fn uniform_1fv(&self, name: &str, values: &[f32]) {
        let location = unsafe { gl::GetUniformLocation(self.program_id, CString::new(name).unwrap().as_ptr()) };
        if location != -1 {
            unsafe {
                gl::Uniform1fv(location, values.len() as i32, values.as_ptr());
            }
        }
    }

    pub fn uniform_2fv(&self, name: &str, value: &Vec2) {
        unsafe {
            let location = gl::GetUniformLocation(self.program_id, CString::new(name).unwrap().as_ptr());
            if location != -1 {
                gl::Uniform2fv(location, 1, value.as_slice().as_ptr());
            }
        }
    }

    pub fn uniform_3fv(&self, name: &str, value: &Vec3) {
        unsafe {
            let location = gl::GetUniformLocation(self.program_id, CString::new(name).unwrap().as_ptr());
            if location != -1 {
                gl::Uniform3fv(location, 1, value.as_slice().as_ptr());
            }
        }
    }

    pub fn uniform_4fv(&self, name: &str, value: &Vec4) {
        unsafe {
            let location = gl::GetUniformLocation(self.program_id, CString::new(name).unwrap().as_ptr());
            if location != -1 {
                gl::Uniform4fv(location, 1, value.as_slice().as_ptr());
            }
        }
    }

    pub fn uniform_matrix_2fv(&self, name: &str, value: &Mat2) {
        unsafe {
            let location = gl::GetUniformLocation(self.program_id, CString::new(name).unwrap().as_ptr());
            if location != -1 {
                gl::UniformMatrix2fv(location, 1, gl::FALSE, value.as_slice().as_ptr());
            }
        }
    }

    pub fn uniform_matrix_3fv(&self, name: &str, value: &Mat3) {
        unsafe {
            let location = gl::GetUniformLocation(self.program_id, CString::new(name).unwrap().as_ptr());
            if location != -1 {
                gl::UniformMatrix3fv(location, 1, gl::FALSE, value.as_slice().as_ptr());
            }
        }
    }

    pub fn uniform_matrix_4fv(&self, name: &str, value: &Mat4) {
        unsafe {
            let location = gl::GetUniformLocation(self.program_id, CString::new(name).unwrap().as_ptr());
            if location != -1 {
                gl::UniformMatrix4fv(location, 1, gl::FALSE, value.as_slice().as_ptr());
            }
        }
    }
}

impl Drop for OpenGLShader {
    fn drop(&mut self) {
        unsafe {
            gl::DetachShader(self.program_id, self.vertex_shader);
            gl::DetachShader(self.program_id, self.fragment_shader);
            gl::DeleteShader(self.vertex_shader);
            gl::DeleteShader(self.fragment_shader);
            gl::DeleteProgram(self.program_id);
        }
    }
}