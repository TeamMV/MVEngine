use alloc::ffi::CString;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::ops::DerefMut;
use std::rc::Rc;

use gl::{COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT};
use gl::types::{GLenum, GLint, GLsizei, GLsizeiptr, GLuint};
use glam::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};
use glfw::ffi::{CLIENT_API, DECORATED, FALSE, glfwCreateWindow, glfwDefaultWindowHints, glfwDestroyWindow, glfwGetPrimaryMonitor, glfwGetProcAddress, glfwGetVideoMode, glfwGetWindowPos, glfwMakeContextCurrent, glfwPollEvents, glfwSetWindowMonitor, glfwSetWindowShouldClose, glfwSetWindowSizeCallback, glfwShowWindow, glfwSwapBuffers, glfwSwapInterval, GLFWwindow, glfwWindowHint, glfwWindowShouldClose, OPENGL_API, RESIZABLE, TRUE, VISIBLE};
use mvutils::utils::{AsCStr, TetrahedronOp, Time};
use once_cell::unsync::Lazy;

use crate::assets::{ReadableAssetManager, SemiAutomaticAssetManager};
use crate::render::{EFFECT_VERT, EMPTY_EFFECT_FRAG, glfwFreeCallbacks};
use crate::render::batch::batch_layout_2d;
use crate::render::camera::{Camera};
use crate::render::draw::Draw2D;
use crate::render::shared::{ApplicationLoop, EffectShader, RenderProcessor2D, Shader, ShaderPassInfo, Texture, Window, WindowCreateInfo};

static mut GL_WINDOWS: Lazy<HashMap<*mut GLFWwindow, *mut OpenGLWindow>> = Lazy::new(HashMap::new);

macro_rules! static_listener {
    ($name:ident, $inner:ident, $($params:ident: $types:ty),+) => {
        extern "C" fn $name(window: *mut GLFWwindow, $($params: $types),+) {
            unsafe {
                let window = GL_WINDOWS.get_mut(&window);
                if let Some(window) = window {
                    window.as_mut().unwrap().$inner($($params),+);
                }
            }
        }
    };
}

static_listener!(res, resize, w: i32, h: i32);

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
    texture_buf: u32,

    camera: Camera,
}

impl OpenGLWindow {
    fn init(&mut self) {
        unsafe {
            glfwDefaultWindowHints();
            glfwWindowHint(VISIBLE, FALSE);
            glfwWindowHint(CLIENT_API, OPENGL_API);
            glfwWindowHint(DECORATED, self.info.decorated.yn(TRUE, FALSE));
            glfwWindowHint(RESIZABLE, self.info.resizable.yn(TRUE, FALSE));

            self.window = glfwCreateWindow(self.info.width, self.info.height, self.info.title.as_c_str().as_ptr(), std::ptr::null_mut(), std::ptr::null_mut());
            GL_WINDOWS.insert(self.window, self);

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

            glfwSetWindowSizeCallback(self.window, Some(res));
        }
    }

    fn running(&mut self, application_loop: &impl ApplicationLoop) {
        unsafe {
            let mut init_time: u128 = u128::time_nanos();
            let mut current_time: u128;
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
                    gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
                    gl::Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
                    //draws
                    self.draw_2d.as_mut().unwrap().reset_canvas();

                    application_loop.draw(self);

                    let len = self.enabled_shaders.len();

                    if len > 0 {
                        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.frame_buf);
                        gl::Clear(COLOR_BUFFER_BIT);
                        self.render_2d.set_framebuffer(self.frame_buf);
                    } else {
                        self.render_2d.set_framebuffer(0);
                    }

                    //TODO: camera 2D and 3D obj so no clone (Rc<RefCell<Camera>>)
                    self.render_2d.set_camera(self.camera.clone());

                    self.draw_2d.as_mut().unwrap().render(&self.render_2d);

                    if len > 0 {
                        for (i, info) in self.enabled_shaders.drain(..).enumerate() {
                            let mut shader = self.shaders.get(info.get_id());
                            if shader.is_none() {
                                if len == i + 1 {
                                    shader = self.shaders.get("empty");
                                } else {
                                    continue;
                                }
                            }
                            let shader = shader.unwrap();
                            shader.borrow_mut().bind();
                            info.apply(shader.borrow_mut().deref_mut());
                            let f_buf = (len == i + 1).yn(0, self.frame_buf);
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

    fn gen_render_buffer(&mut self) {
        unsafe {
            if self.frame_buf != 0 {
                gl::DeleteBuffers(1, &self.frame_buf);
            }

            gl::CreateFramebuffers(1, &mut self.frame_buf);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.frame_buf);

            gl::GenTextures(1, &mut self.texture_buf);
            gl::BindTexture(gl::TEXTURE_2D, self.texture_buf);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as GLint, self.info.width, self.info.height, 0, gl::RGB, gl::UNSIGNED_BYTE, std::ptr::null());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::BindFramebuffer(gl::FRAMEBUFFER, self.frame_buf);

            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, self.texture_buf, 0);

            let attach = [gl::COLOR_ATTACHMENT0];
            gl::DrawBuffers(1, attach.as_ptr());

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("Incomplete Framebuffer");
            }

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            self.info.width = width;
            self.info.height = height;
            gl::Viewport(0, 0, width, height);
            self.gen_render_buffer();
            self.render_2d.resize(width, height);
            self.draw_2d.as_mut().unwrap().resize(width, height);
            self.shader_pass.resize(width, height);
            self.camera.update_projection_mat(width, height);
        }
    }
}

impl Window for OpenGLWindow {
    fn new(info: WindowCreateInfo, assets: Rc<RefCell<SemiAutomaticAssetManager>>) -> Self {
        OpenGLWindow {
            info,
            assets,
            window: std::ptr::null_mut(),
            current_fps: 0,
            current_ups: 0,
            current_frame: 0,

            size_buf: [0; 4],
            render_2d: OpenGLRenderProcessor2D::new(),
            shaders: HashMap::new(),
            enabled_shaders: Vec::with_capacity(10),
            shader_pass: OpenGLShaderPass::new(),
            frame_buf: 0,
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

        self.gen_render_buffer();
        self.shader_pass.resize(self.info.width, self.info.height);
        self.render_2d.resize(self.info.width, self.info.height);
        let shader = self.assets.borrow_mut().get_shader("default");
        self.draw_2d = Some(Draw2D::new(shader, self.info.width, self.info.height));

        unsafe {
            self.shaders.insert("empty".to_string(), Rc::new(RefCell::new(EffectShader::OpenGL(OpenGLShader::new(EFFECT_VERT, EMPTY_EFFECT_FRAG)))));
        }

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

    fn get_width(&self) -> i32 {
        self.info.width
    }

    fn get_height(&self) -> i32 {
        self.info.height
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
                self.size_buf[2] = self.info.width;
                self.size_buf[3] = self.info.height;
                let monitor = glfwGetPrimaryMonitor();
                let mode = glfwGetVideoMode(monitor);
                glfwSetWindowMonitor(self.window, monitor, 0, 0, (*mode).width, (*mode).height, (*mode).refreshRate);
                //self.info.width = (*mode).width as u16;
                //self.info.height = (*mode).height as u16;
                //gl::Viewport(0, 0, (*mode).width, (*mode).height);
            } else {
                let monitor = glfwGetPrimaryMonitor();
                let mode = glfwGetVideoMode(monitor);
                glfwSetWindowMonitor(self.window, std::ptr::null_mut(), self.size_buf[0], self.size_buf[1], self.size_buf[2], self.size_buf[3], (*mode).refreshRate);
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
    width: i32,
    height: i32,
}

impl OpenGLShaderPass {
    pub fn new() -> Self {
        Self {
            ibo: 0,
            indices: [0, 2, 1, 1, 2, 3],
            width: 0,
            height: 0,
        }
    }

    pub fn set_ibo(&mut self, ibo: u32) {
        self.ibo = ibo;
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    pub fn render(&self, shader: &mut EffectShader, f_buf: u32, t_buf: u32, frame: i32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, t_buf);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, 24, self.indices.as_ptr() as *const _, gl::STATIC_DRAW);

            gl::BindFramebuffer(gl::FRAMEBUFFER, f_buf);

            shader.uniform_1i("tex", 0);
            shader.uniform_2fv("res", Vec2::new(self.width as f32, self.height as f32));
            shader.uniform_1f("time", frame as f32);

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
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

    pub(crate) unsafe fn uniform_fv(&self, name: &str, value: &[f32]) {
        shader_uniform!(Uniform1fv, self.prgm_id, name, value.len() as i32, value.as_ptr());
    }

    pub(crate) unsafe fn uniform_iv(&self, name: &str, value: &[i32]) {
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
    gl_id: u32,
}

impl OpenGLTexture {
    pub(crate) unsafe fn new(bytes: Vec<u8>) -> Self {
        OpenGLTexture {
            bytes,
            width: 0,
            height: 0,
            gl_id: 0,
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
    camera: Option<Camera>,
}

impl OpenGLRenderProcessor2D {
    fn new() -> Self {
        OpenGLRenderProcessor2D {
            framebuffer: 0,
            width: 0,
            height: 0,
            camera: None,
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
    #[allow(clippy::too_many_arguments)]
    fn process_data(&self, tex: &mut [Option<Rc<RefCell<Texture>>>], tex_id: &[u32], indices: &[u32], vertices: &[f32], vbo: u32, ibo: u32, shader: &mut Shader, render_mode: u8) {
        let mut i: u8 = 0;
        for t in tex.iter_mut().flatten() {
            t.borrow_mut().bind(i);
            i += 1;
        }

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * 4) as GLsizeiptr, vertices.as_ptr() as *const c_void, gl::DYNAMIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (indices.len() * 4) as GLsizeiptr, indices.as_ptr() as *const c_void, gl::DYNAMIC_DRAW);

            if !tex.is_empty() {
                shader.uniform_iv("TEX_SAMPLER", tex_id.iter().map(|u| { *u as i32 }).collect::<Vec<_>>().as_slice());
            }

            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);

            shader.uniform_1i("uResX", self.width);
            shader.uniform_1i("uResY", self.height);
            shader.uniform_4fm("uProjection", self.camera.as_ref().unwrap().get_projection_mat());
            shader.uniform_4fm("uView", self.camera.as_ref().unwrap().get_view_mat());

            vert_attrib!(0, POSITION_SIZE, POSITION_OFFSET_BYTES);
            vert_attrib!(1, ROTATION_SIZE, ROTATION_OFFSET_BYTES);
            vert_attrib!(2, ROTATION_ORIGIN_SIZE, ROTATION_ORIGIN_OFFSET_BYTES);
            vert_attrib!(3, COLOR_SIZE, COLOR_OFFSET_BYTES);
            vert_attrib!(4, UV_SIZE, UV_OFFSET_BYTES);
            vert_attrib!(5, TEX_ID_SIZE, TEX_ID_OFFSET_BYTES);
            vert_attrib!(6, CANVAS_COORDS_SIZE, CANVAS_COORDS_OFFSET_BYTES);
            vert_attrib!(7, CANVAS_DATA_SIZE, CANVAS_DATA_OFFSET_BYTES);
            vert_attrib!(8, USE_CAMERA_SIZE, USE_CAMERA_OFFSET_BYTES);

            gl::DrawElements(render_mode as GLenum, indices.len() as GLsizei, gl::UNSIGNED_INT, std::ptr::null());

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        for t in tex.iter_mut().flatten() {
            t.borrow_mut().unbind();
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