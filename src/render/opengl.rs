use alloc::ffi::CString;
use std::ffi::{c_int, c_void, CStr};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::mem;
use std::ops::{Deref, DerefMut};

use gl::{COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT, FLOAT, FRAMEBUFFER_BARRIER_BIT};
use gl::types::{GLboolean, GLdouble, GLenum, GLint, GLsizei, GLsizeiptr, GLuint};
use glam::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};
use glfw::{Context, Glfw, WindowMode};
use glfw::ClientApiHint::OpenGl;
use glfw::ffi::{CLIENT_API, DECORATED, FALSE, glfwCreateWindow, glfwDefaultWindowHints, glfwDestroyWindow, glfwGetPrimaryMonitor, glfwGetProcAddress, glfwGetVideoMode, glfwGetWindowPos, glfwInit, glfwMakeContextCurrent, glfwPollEvents, glfwSetCharCallback, glfwSetCharModsCallback, glfwSetCursorEnterCallback, glfwSetCursorPosCallback, glfwSetDropCallback, glfwSetFramebufferSizeCallback, glfwSetKeyCallback, glfwSetMouseButtonCallback, glfwSetScrollCallback, glfwSetWindowCloseCallback, glfwSetWindowContentScaleCallback, glfwSetWindowFocusCallback, glfwSetWindowIconifyCallback, glfwSetWindowMaximizeCallback, glfwSetWindowMonitor, glfwSetWindowPosCallback, glfwSetWindowRefreshCallback, glfwSetWindowShouldClose, glfwSetWindowSizeCallback, glfwShowWindow, glfwSwapBuffers, glfwSwapInterval, GLFWwindow, glfwWindowHint, glfwWindowShouldClose, OPENGL_API, RESIZABLE, TRUE, VISIBLE};
use glfw::WindowHint::{ClientApi, Decorated, Resizable, Visible};
use glfw::WindowMode::Windowed;
use mvutils::utils::{AsCStr, IncDec, TetrahedronOp, Time};

use crate::assets;
use crate::assets::{ReadableAssetManager, SemiAutomaticAssetManager};
use crate::render::batch::batch_layout_2d;
use crate::render::batch::batch_layout_2d::{POSITION_OFFSET_BYTES, POSITION_SIZE, VERTEX_SIZE_BYTES};
use crate::render::camera::{Camera, Camera2D};
use crate::render::draw::Draw2D;
use crate::render::glfwFreeCallbacks;
use crate::render::shared::{ApplicationLoop, EffectShader, RenderProcessor2D, Shader, ShaderPassInfo, Texture, Window, WindowCreateInfo};

pub struct OpenGLWindow {
    info: WindowCreateInfo,
    assets: Rc<RefCell<SemiAutomaticAssetManager>>,
    window: *mut GLFWwindow,
    current_fps: u16,
    current_ups: u16,
    current_frame: u128,

    size_buf: [i32; 4],

    draw_2d: Option<Draw2D>,
    render_2d: OpenGLRenderProcessor2D,
    shaders: HashMap<String, Rc<RefCell<EffectShader>>>,
    enabled_shaders: Vec<ShaderPassInfo>,
    shader_pass: OpenGLShaderPass,
    frame_buf: u32,
    render_buf: u32,
    texture_buf: u32,

    camera: Camera
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
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthMask(gl::TRUE);
            gl::DepthFunc(gl::LEQUAL);

            self.shader_pass.set_ibo(self.render_2d.gen_buffer_id());
            self.gen_render_buffer();
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
                        gl::ClearTexImage(self.texture_buf, 0, gl::RGB, gl::FLOAT, 0 as *const c_void);
                    }
                    //draws

                    application_loop.draw(self);

                    let len = self.enabled_shaders.len();

                    self.render_2d.set_framebuffer((len > 0).yn(self.frame_buf, 0));
                    self.render_2d.set_camera(self.camera.clone());

                    self.draw_2d.as_mut().unwrap().render(&mut self.render_2d);

                    if len > 0 {
                        for (i, info) in self.enabled_shaders.drain(0..).into_iter().enumerate() {
                            let shader = self.shaders.get(info.get_id());
                            if shader.is_none() {
                                continue;
                            }
                            let shader = shader.unwrap();
                            info.apply(shader.borrow_mut().deref_mut());
                            let f_buf = (len - i == 1).yn(0, self.frame_buf);
                            self.shader_pass.render(shader.borrow_mut().deref_mut(), f_buf, self.texture_buf, self.current_frame as i32);
                        }
                    }

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

    fn gen_render_buffer(&mut self) {
        unsafe {
            if self.frame_buf != 0 {
                gl::DeleteFramebuffers(1, &mut self.frame_buf);
            }

            gl::GenFramebuffers(1, &mut self.frame_buf);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.frame_buf);

            if self.texture_buf != 0 {
                gl::DeleteTextures(1, &mut self.texture_buf);
            }

            gl::GenTextures(1, &mut self.texture_buf);
            gl::BindTexture(gl::TEXTURE_2D, self.texture_buf);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as GLint, self.info.width as i32, self.info.height as i32, 0, gl::RGB, gl::FLOAT, 0 as *const _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            //gl::BindTexture(gl::TEXTURE_2D, 0);

            if self.render_buf != 0 {
                gl::DeleteRenderbuffers(1, &mut self.render_buf);
            }

            gl::GenRenderbuffers(1, &mut self.render_buf);
            gl::BindRenderbuffer(gl::RENDERBUFFER, self.render_buf);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT32, self.info.width as i32, self.info.height as i32);
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, self.render_buf);
            //gl::BindRenderbuffer(gl::RENDERBUFFER, 0);

            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, self.texture_buf, 0);

            let attachments = [gl::COLOR_ATTACHMENT0];
            gl::DrawBuffers(1, attachments.as_ptr());

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("Incomplete Framebuffer");
            }

            //gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
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

            size_buf: [0; 4],
            render_2d: OpenGLRenderProcessor2D::new(),
            shaders: HashMap::new(),
            enabled_shaders: Vec::with_capacity(10),
            shader_pass: OpenGLShaderPass::new(),
            frame_buf: 0,
            render_buf: 0,
            texture_buf: 0,

            draw_2d: None,
            camera: Camera::new_2d(),
        }
    }

    fn run(&mut self, application_loop: impl ApplicationLoop) {
        self.init();

        if self.info.fullscreen {
            self.set_fullscreen(true);
        }

        self.shader_pass.resize(self.info.width, self.info.height);
        self.render_2d.resize(self.info.width as i32, self.info.height as i32);
        let shader = self.assets.borrow_mut().get_shader("default");
        self.draw_2d = Some(Draw2D::new(shader, self.info.width, self.info.height));

        self.camera.update_projection_mat(self.info.width, self.info.height);
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

    fn get_draw_2d(&mut self) -> &mut Draw2D {
        self.draw_2d.as_mut().expect("The Draw2D is not initialized yet!")
    }

    fn set_fullscreen(&mut self, fullscreen: bool) {
        unsafe {
            self.info.fullscreen = fullscreen;
            if fullscreen {
                glfwGetWindowPos(self.window, &mut self.size_buf[0] as *mut _, &mut self.size_buf[1] as *mut _);
                self.size_buf[3] = self.info.width as i32;
                self.size_buf[4] = self.info.height as i32;
                let monitor = glfwGetPrimaryMonitor();
                let mode = glfwGetVideoMode(monitor);
                glfwSetWindowMonitor(self.window, monitor, 0, 0, (*mode).width, (*mode).height, (*mode).refreshRate);
                self.info.width = (*mode).width as u16;
                self.info.height = (*mode).height as u16;
            } else {
                let monitor = glfwGetPrimaryMonitor();
                let mode = glfwGetVideoMode(monitor);
                glfwSetWindowMonitor(self.window, 0 as *mut _, self.size_buf[0], self.size_buf[1], self.size_buf[2], self.size_buf[3], (*mode).refreshRate);
            }
        }
    }

    fn get_glfw_window(&self) -> *mut GLFWwindow {
        self.window
    }

    fn add_shader(&mut self, id: &str, shader: Rc<RefCell<EffectShader>>) {
        self.shaders.insert(id.to_string(), shader);
    }

    fn queue_shader_pass(&mut self, info: ShaderPassInfo) {
        self.enabled_shaders.push(info);
    }

    fn get_camera(&self) -> &Camera {
        &self.camera
    }
}

struct OpenGLShaderPass {
    ibo: u32,
    indices: [u32; 6],
    width: u16,
    height: u16
}

impl OpenGLShaderPass {
    pub fn new() -> Self {
        Self {
            ibo: 0,
            indices: [0, 2, 1, 1, 2, 3],
            width: 0,
            height: 0
        }
    }

    pub fn set_ibo(&mut self, ibo: u32) {
        self.ibo = ibo;
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    pub fn render(&self, shader: &mut EffectShader, f_buf: u32, t_buf: u32, frame: i32) {
        shader.bind();
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, t_buf);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, 24, self.indices.as_ptr() as *const _, gl::STATIC_DRAW);

            gl::BindFramebuffer(gl::FRAMEBUFFER, f_buf);

            //TODO: shader uniforms
            shader.uniform_1i("tex", gl::TEXTURE0 as i32);
            shader.uniform_2fv("res", Vec2::new(self.width as f32, self.height as f32));
            shader.uniform_1f("time", frame as f32);

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
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
        if self.prgm_id != 0 {
            return;
        }
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

    pub(crate) unsafe fn bind(&mut self) {
        if self.prgm_id == 0 {
            self.make();
        }
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

    pub(crate) unsafe fn uniform_2fv(&self, name: &str, value: Vec2) {
        shader_uniform!(Uniform2fv, self.prgm_id, name, 1, value.to_array().as_ptr());
    }

    pub(crate) unsafe fn uniform_3fv(&self, name: &str, value: Vec3) {
        shader_uniform!(Uniform3fv, self.prgm_id, name, 1, value.to_array().as_ptr());
    }

    pub(crate) unsafe fn uniform_4fv(&self, name: &str, value: Vec4) {
        shader_uniform!(Uniform4fv, self.prgm_id, name, 1, value.to_array().as_ptr());
    }

    pub(crate) unsafe fn uniform_2fm(&self, name: &str, value: Mat2) {
        shader_uniform!(UniformMatrix2fv, self.prgm_id, name, 1, 0, value.to_cols_array().as_ptr());
    }

    pub(crate) unsafe fn uniform_3fm(&self, name: &str, value: Mat3) {
        shader_uniform!(UniformMatrix3fv, self.prgm_id, name, 1, 0, value.to_cols_array().as_ptr());
    }

    pub(crate) unsafe fn uniform_4fm(&self, name: &str, value: Mat4) {
        shader_uniform!(UniformMatrix4fv, self.prgm_id, name, 1, 0, value.to_cols_array().as_ptr());
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
        if self.gl_id != 0 {
            return;
        }
        gl::GenTextures(1, &mut self.gl_id);
        gl::BindTexture(gl::TEXTURE_2D, self.gl_id);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, self.width as GLsizei, self.height as GLsizei, 0, gl::RGBA, gl::UNSIGNED_BYTE, self.bytes.as_ptr() as *const c_void);
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }

    pub(crate) unsafe fn bind(&mut self, index: u8) {
        if self.gl_id == 0 {
            self.make();
        }
        gl::ActiveTexture(gl::TEXTURE0 + index as u32);
        gl::BindTexture(gl::TEXTURE_2D, self.gl_id);
    }

    pub(crate) unsafe fn unbind(&mut self) {
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

pub(crate) struct OpenGLRenderProcessor2D {
    framebuffer: u32,
    width: i32,
    height: i32,
    camera: Option<Camera>
}

impl OpenGLRenderProcessor2D {
    fn new() -> Self {
        OpenGLRenderProcessor2D {
            framebuffer: 0,
            width: 0,
            height: 0,
            camera: None
        }
    }

    fn resize(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    fn set_framebuffer(&mut self, framebuffer: u32) {
        self.framebuffer = framebuffer;
    }

    fn set_camera(&mut self, cam: Camera) {
        self.camera = Some(cam);
    }
}

macro_rules! vert_attrib {
    ($idx:expr, $size:ident, $off:ident) => {
        gl::VertexAttribPointer($idx, batch_layout_2d::$size as GLint, gl::FLOAT, 0, batch_layout_2d::VERTEX_SIZE_BYTES as GLsizei, batch_layout_2d::$off as *const _);
        gl::EnableVertexAttribArray($idx);
    };
}

impl RenderProcessor2D for OpenGLRenderProcessor2D<> {
    fn process_data(&self, tex: &mut [Option<Rc<RefCell<Texture>>>], tex_id: &[u32], indices: &Vec<u32>, vertices: &Vec<f32>, vbo: u32, ibo: u32, shader: &mut Shader, render_mode: u8) {
        let mut i: u8 = 0;
        for op in tex.iter_mut() {
            if let Some(t) = op {
                t.borrow_mut().bind(i);
                i += 1;
            }
        }

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * 4) as GLsizeiptr, vertices.as_ptr() as *const c_void, gl::DYNAMIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (indices.len() * 4) as GLsizeiptr, indices.as_ptr() as *const c_void, gl::DYNAMIC_DRAW);

            if !tex.is_empty() {
                shader.uniform_iv("TEX_SAMPLER", &tex_id.clone().into_iter().map(|u| {u.clone() as i32}).collect::<Vec<_>>());
            }

            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);

            shader.uniform_1i("uResX", self.width);
            shader.uniform_1i("uResY", self.height);
            shader.uniform_4fm("uProjection", self.camera.as_ref().unwrap().get_projection_mat().clone());
            shader.uniform_4fm("uView", self.camera.as_ref().unwrap().get_view_mat().clone());

            vert_attrib!(0, POSITION_SIZE, POSITION_OFFSET_BYTES);
            vert_attrib!(1, ROTATION_SIZE, ROTATION_OFFSET_BYTES);
            vert_attrib!(2, ROTATION_ORIGIN_SIZE, ROTATION_ORIGIN_OFFSET_BYTES);
            vert_attrib!(3, COLOR_SIZE, COLOR_OFFSET_BYTES);
            vert_attrib!(4, UV_SIZE, UV_OFFSET_BYTES);
            vert_attrib!(5, TEX_ID_SIZE, TEX_ID_OFFSET_BYTES);
            vert_attrib!(6, CANVAS_COORDS_SIZE, CANVAS_COORDS_OFFSET_BYTES);
            vert_attrib!(7, CANVAS_DATA_SIZE, CANVAS_DATA_OFFSET_BYTES);
            vert_attrib!(8, USE_CAMERA_SIZE, USE_CAMERA_OFFSET_BYTES);

            gl::DrawElements(render_mode as GLenum, indices.len() as GLsizei, gl::UNSIGNED_INT, 0 as *const _);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        for op in tex.iter_mut() {
            if let Some(t) = op {
                t.borrow_mut().unbind();
            }
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