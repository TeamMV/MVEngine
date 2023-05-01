use alloc::ffi::CString;
use std::cell::RefCell;
use std::cmp::min;
use std::collections::HashMap;
use std::ffi::{c_float, c_void};
use std::io::Cursor;
use std::ops::DerefMut;
use std::rc::Rc;

use gl::{COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT};
use gl::types::{GLenum, GLint, GLsizei, GLsizeiptr, GLuint};
use glam::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};
use glfw::ffi::{CLIENT_API, DECORATED, FALSE, glfwCreateWindow, glfwDefaultWindowHints, glfwDestroyWindow, glfwGetMonitorContentScale, glfwGetMonitorPhysicalSize, glfwGetPrimaryMonitor, glfwGetProcAddress, glfwGetVideoMode, glfwGetWindowPos, glfwMakeContextCurrent, glfwPollEvents, glfwSetWindowMonitor, glfwSetWindowShouldClose, glfwSetWindowSizeCallback, glfwShowWindow, glfwSwapBuffers, glfwSwapInterval, GLFWwindow, glfwWindowHint, glfwWindowShouldClose, OPENGL_API, RESIZABLE, TRUE, VISIBLE};
use mvutils::utils::{AsCStr, RcMut, TetrahedronOp, Time};
use once_cell::sync::Lazy;

use crate::assets::{ReadableAssetManager, SemiAutomaticAssetManager};
use crate::render::{glfwFreeCallbacks, load_render_assets, shader_preprocessor};
use crate::render::batch2d::batch_layout_2d;
use crate::render::camera::{Camera2D, Camera3D};
use crate::render::draw::Draw2D;
use crate::render::shared::{ApplicationLoop, EffectShader, RenderProcessor2D, RunningWindow, Shader, ShaderPassInfo, Texture, WindowCreateInfo};

#[cfg(feature = "3d")]
use crate::render::opengl::deferred::{OpenGLGeometryPass, OpenGLLightingPass};
#[cfg(feature = "3d")]
use crate::render::model::Material;
use crate::render::shader_preprocessor::{MAX_NUM_LIGHTS, MAX_TEXTURES, process, TEXTURE_LIMIT};
#[cfg(feature = "3d")]
#[cfg(feature = "3d")]
use crate::render::lights::Light;
use crate::render::shared::RenderProcessor3D;

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

pub(super) fn gen_buffer_id() -> u32 {
    unsafe {
        let mut buf: GLuint = 0;
        gl::GenBuffers(1, &mut buf);
        buf
    }
}

pub struct OpenGLWindow {
    info: WindowCreateInfo,
    assets: Rc<RefCell<SemiAutomaticAssetManager>>,
    window: *mut GLFWwindow,
    current_fps: u16,
    current_ups: u16,
    current_frame: u64,

    size_buf: [i32; 4],

    draw_2d: Option<RcMut<Draw2D>>,
    render_2d: OpenGLRenderProcessor2D,

    #[cfg(feature = "3d")]
    forward_render: OpenGLRenderProcessor3D,
    #[cfg(feature = "3d")]
    geometry_pass: OpenGLGeometryPass,
    #[cfg(feature = "3d")]
    lighting_pass: OpenGLLightingPass,

    shaders: HashMap<String, Rc<RefCell<EffectShader>>>,
    enabled_shaders: Vec<ShaderPassInfo>,
    shader_pass: OpenGLShaderPass,
    shader_buffer_2d: OpenGLShaderBuffer,
    #[cfg(feature = "3d")]
    shader_buffer_3d: OpenGLShaderBuffer,

    camera_2d: Camera2D,
    camera_3d: Camera3D,

    dpi: f32
}

impl OpenGLWindow {
    pub(crate) fn new(info: WindowCreateInfo, assets: Rc<RefCell<SemiAutomaticAssetManager>>) -> Self {
        OpenGLWindow {
            info,
            assets,

            window: std::ptr::null_mut(),

            current_fps: 0,
            current_ups: 0,
            current_frame: 0,

            size_buf: [0; 4],
            render_2d: OpenGLRenderProcessor2D::new(),

            #[cfg(feature = "3d")]
            forward_render: OpenGLRenderProcessor3D::new(),
            #[cfg(feature = "3d")]
            geometry_pass: OpenGLGeometryPass::new(),
            #[cfg(feature = "3d")]
            lighting_pass: OpenGLLightingPass::new(),

            shaders: HashMap::new(),
            enabled_shaders: Vec::with_capacity(10),
            shader_pass: OpenGLShaderPass::new(),
            shader_buffer_2d: OpenGLShaderBuffer::default(),
            #[cfg(feature = "3d")]
            shader_buffer_3d: OpenGLShaderBuffer::default(),

            draw_2d: None,
            camera_2d: Camera2D::default(),
            camera_3d: Camera3D::default(),
            dpi: 0.0
        }
    }

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

            self.shader_pass.set_ibo(gen_buffer_id());

            let mut mt = 0;
            gl::GetIntegerv(gl::MAX_TEXTURE_IMAGE_UNITS, &mut mt);
            MAX_TEXTURES = min(mt as u32, TEXTURE_LIMIT);
            gl::GetIntegerv(gl::MAX_UNIFORM_BUFFER_BINDINGS, &mut mt);
            MAX_NUM_LIGHTS = mt as u32;

            glfwSetWindowSizeCallback(self.window, Some(res));

            let vidm = glfwGetVideoMode(glfwGetPrimaryMonitor()).as_ref().unwrap();
            let pxw = vidm.width as f32;

            let mut mon_phy_x: i32 = 1;
            let mut mon_phy_y: i32 = 1;
            glfwGetMonitorPhysicalSize(glfwGetPrimaryMonitor(), &mut mon_phy_x as *mut _, &mut mon_phy_y as *mut _);

            let mut scly: f32 = 0.0;
            let mut sclx: f32 = 0.0;
            glfwGetMonitorContentScale(glfwGetPrimaryMonitor(), &mut sclx as *mut _, &mut scly as *mut _);

            self.dpi = pxw / (mon_phy_x as f32 / 25.4) * sclx;
        }
    }

    fn running(&mut self, application_loop: &mut impl ApplicationLoop) {
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

                    application_loop.update(RunningWindow::OpenGL(self));
                    ticks += 1;
                    delta_u -= 1.0;
                }
                if delta_f >= 1.0 {
                    gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
                    gl::Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
                    //draws

                    self.draw_2d.clone().unwrap().borrow_mut().reset_canvas();

                    application_loop.draw(RunningWindow::OpenGL(self));

                    #[cfg(feature = "3d")]
                    self.render_3d();

                    self.render_2d();

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

    unsafe fn render_2d(&mut self) {
        let len = self.enabled_shaders.len();

        if len > 0 {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.shader_buffer_2d.f_buf);
            gl::Clear(COLOR_BUFFER_BIT);
            self.render_2d.set_framebuffer(self.shader_buffer_2d.f_buf);
        } else {
            self.render_2d.set_framebuffer(0);
        }

        //TODO: camera 2D and 3D obj so no clone (Rc<RefCell<Camera>>)
        self.render_2d.set_camera(self.camera_2d.clone());

        self.draw_2d.clone().unwrap().borrow_mut().render(&self.render_2d);

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
                let f_buf = (len == i + 1).yn(0, self.shader_buffer_2d.f_buf);
                self.shader_pass.render(shader.borrow_mut().deref_mut(), f_buf, self.shader_buffer_2d.t_buf, self.current_frame as i32);
            }
        }
    }

    #[cfg(feature = "3d")]
    unsafe fn render_3d(&mut self) {

    }

    fn terminate(&mut self) {
        unsafe {
            GL_WINDOWS.remove(&self.window);
            glfwFreeCallbacks(self.window);
            glfwDestroyWindow(self.window);
        }
    }

    fn gen_render_buffers(&mut self) {
        unsafe {
            if self.shader_buffer_2d.f_buf != 0 {
                gl::DeleteFramebuffers(1, &self.shader_buffer_2d.f_buf);
            }
            if self.shader_buffer_2d.t_buf != 0 {
                gl::DeleteTextures(1, &self.shader_buffer_2d.t_buf);
            }

            gl::CreateFramebuffers(1, &mut self.shader_buffer_2d.f_buf);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.shader_buffer_2d.f_buf);

            gl::GenTextures(1, &mut self.shader_buffer_2d.t_buf);
            gl::BindTexture(gl::TEXTURE_2D, self.shader_buffer_2d.t_buf);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as GLint, self.info.width, self.info.height, 0, gl::RGB, gl::UNSIGNED_BYTE, std::ptr::null());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::BindFramebuffer(gl::FRAMEBUFFER, self.shader_buffer_2d.f_buf);

            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, self.shader_buffer_2d.t_buf, 0);

            let attach = [gl::COLOR_ATTACHMENT0];
            gl::DrawBuffers(1, attach.as_ptr());

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("Incomplete 2D Framebuffer!");
            }

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        #[cfg(feature = "3d")]
        unsafe {
            if self.shader_buffer_3d.f_buf != 0 {
                gl::DeleteFramebuffers(1, &self.shader_buffer_3d.f_buf);
            }
            if self.shader_buffer_3d.t_buf != 0 {
                gl::DeleteTextures(1, &self.shader_buffer_3d.t_buf);
            }
            if self.shader_buffer_3d.r_buf != 0 {
                gl::DeleteRenderbuffers(1, &self.shader_buffer_3d.r_buf);
            }

            gl::CreateFramebuffers(1, &mut self.shader_buffer_3d.f_buf);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.shader_buffer_3d.f_buf);

            gl::GenTextures(1, &mut self.shader_buffer_3d.t_buf);
            gl::BindTexture(gl::TEXTURE_2D, self.shader_buffer_3d.t_buf);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as GLint, self.info.width, self.info.height, 0, gl::RGB, gl::UNSIGNED_BYTE, std::ptr::null());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::GenRenderbuffers(1, &mut self.shader_buffer_3d.r_buf);
            gl::BindRenderbuffer(gl::RENDERBUFFER, self.shader_buffer_3d.r_buf);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT, self.info.width, self.info.height);

            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, self.shader_buffer_3d.t_buf, 0);
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, self.shader_buffer_3d.r_buf);

            let attach = [gl::COLOR_ATTACHMENT0];
            gl::DrawBuffers(1, attach.as_ptr());

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("Incomplete 3D Framebuffer!");
            }

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            self.info.width = width;
            self.info.height = height;
            gl::Viewport(0, 0, width, height);
            self.gen_render_buffers();
            self.render_2d.resize(width, height);
            #[cfg(feature = "3d")]
            self.geometry_pass.resize(width, height);
            self.draw_2d.clone().unwrap().borrow_mut().resize(width, height);
            self.shader_pass.resize(width, height);
            self.camera_2d.update_projection_mat(width, height);
            self.camera_3d.update_projection_mat(width, height);
        }
    }

    pub(crate) fn run(&mut self, mut application_loop: impl ApplicationLoop) {
        self.init();

        if self.info.fullscreen {
            self.set_fullscreen(true);
        }

        load_render_assets(self.assets.clone());

        self.gen_render_buffers();
        self.shader_pass.resize(self.info.width, self.info.height);
        self.render_2d.setup(self.info.width, self.info.height);
        let shader = self.assets.borrow().get_shader("default");
        let font = self.assets.borrow().get_font("default");
        self.draw_2d = Some(RcMut::new(RefCell::new(Draw2D::new(shader, font, self.info.width, self.info.height, self.get_dpi()))));

        #[cfg(feature = "3d")]
        {
            self.geometry_pass.setup(self.info.width, self.info.height);
            self.lighting_pass.setup();
            self.forward_render.setup(self.info.width, self.info.height);
        }

        self.camera_2d.update_projection_mat(self.info.width, self.info.height);
        self.camera_3d.update_projection_mat(self.info.width, self.info.height);
        application_loop.start(RunningWindow::OpenGL(self));

        self.running(&mut application_loop);
        application_loop.stop(RunningWindow::OpenGL(self));
        self.terminate();
    }

    pub(crate) fn stop(&mut self) {
        unsafe {
            glfwSetWindowShouldClose(self.window, TRUE);
        }
    }

    pub(crate) fn get_width(&self) -> i32 {
        self.info.width
    }

    pub(crate) fn get_height(&self) -> i32 {
        self.info.height
    }

    pub(crate) fn get_dpi(&self) -> f32 {
        self.dpi
    }

    pub(crate) fn get_fps(&self) -> u16 {
        self.current_fps
    }

    pub(crate) fn get_ups(&self) -> u16 {
        self.current_ups
    }

    pub(crate) fn get_frame(&self) -> u64 {
        self.current_frame
    }

    pub(crate) fn get_draw_2d(&mut self) -> RcMut<Draw2D> {
        self.draw_2d.clone().expect("The Draw2D is not initialized yet!")
    }

    pub(crate) fn set_fullscreen(&mut self, fullscreen: bool) {
        unsafe {
            self.info.fullscreen = fullscreen;
            if fullscreen {
                glfwGetWindowPos(self.window, &mut self.size_buf[0] as *mut _, &mut self.size_buf[1] as *mut _);
                self.size_buf[2] = self.info.width;
                self.size_buf[3] = self.info.height;
                let monitor = glfwGetPrimaryMonitor();
                let mode = glfwGetVideoMode(monitor);
                glfwSetWindowMonitor(self.window, monitor, 0, 0, (*mode).width, (*mode).height, (*mode).refreshRate);
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

    pub(crate) fn add_shader(&mut self, id: &str, shader: Rc<RefCell<EffectShader>>) {
        self.shaders.insert(id.to_string(), shader);
    }

    pub(crate) fn queue_shader_pass(&mut self, info: ShaderPassInfo) {
        self.enabled_shaders.push(info);
    }

    pub(crate) fn get_camera_2d(&self) -> &Camera2D {
        &self.camera_2d
    }

    pub(crate) fn get_camera_3d(&self) -> &Camera3D {
        &self.camera_3d
    }

    pub(crate) fn get_render_3d(&self) -> &OpenGLGeometryPass {
        &self.geometry_pass
    }

    pub(crate) fn get_lighting(&self) -> &OpenGLLightingPass {
        &self.lighting_pass
    }
}

#[derive(Default, Copy, Clone)]
struct OpenGLShaderBuffer {
    f_buf: u32,
    t_buf: u32,
    r_buf: u32,
}

struct OpenGLShaderPass {
    ibo: u32,
    indices: [u32; 6],
    width: i32,
    height: i32,
}

impl OpenGLShaderPass {
    fn new() -> Self {
        Self {
            ibo: 0,
            indices: [0, 2, 1, 1, 2, 3],
            width: 0,
            height: 0,
        }
    }

    fn set_ibo(&mut self, ibo: u32) {
        self.ibo = ibo;
    }

    fn resize(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    fn render(&self, shader: &mut EffectShader, f_buf: u32, t_buf: u32, frame: i32) {
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
    vertex: Option<CString>,
    fragment: Option<CString>,
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
            vertex: Some(CString::new(process(vertex)).unwrap()),
            fragment: Some(CString::new(process(fragment)).unwrap()),
            prgm_id: 0,
            vertex_id: 0,
            fragment_id: 0,
        }
    }

    unsafe fn create_shader(&self, id: GLuint, src: CString) {
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
            panic!("{}: {}", id, log);
        }
    }

    pub(crate) unsafe fn make(&mut self) {
        if self.prgm_id != 0 || self.vertex.is_none() || self.fragment.is_none() {
            return;
        }
        self.prgm_id = gl::CreateProgram();
        self.vertex_id = gl::CreateShader(gl::VERTEX_SHADER);
        self.fragment_id = gl::CreateShader(gl::FRAGMENT_SHADER);
        let vertex = self.vertex.take().unwrap();
        let fragment = self.fragment.take().unwrap();
        self.create_shader(self.vertex_id, vertex);
        self.create_shader(self.fragment_id, fragment);

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

    #[cfg(feature = "3d")]
    pub(crate) unsafe fn uniform_material(&self, name: &str, value: &Material) {
        self.uniform_4fv((name.to_string() + ".ambient").as_str(), value.ambient.as_vec());
        self.uniform_4fv((name.to_string() + ".diffuse").as_str(), value.diffuse.as_vec());
        self.uniform_4fv((name.to_string() + ".specular").as_str(), value.specular.as_vec());
        self.uniform_4fv((name.to_string() + ".emission").as_str(), value.emission.as_vec());
        self.uniform_1f((name.to_string() + ".alpha").as_str(), value.alpha);
        self.uniform_1f((name.to_string() + ".specularExponent").as_str(), value.specular_exponent);
        self.uniform_1f((name.to_string() + ".metallic").as_str(), value.metallic);
        self.uniform_1f((name.to_string() + ".roughness").as_str(), value.roughness);
        self.uniform_1i((name.to_string() + ".diffuseTextureId").as_str(), 0);
        self.uniform_1i((name.to_string() + ".metallicRoughnessTextureId").as_str(), 0);
        self.uniform_1i((name.to_string() + ".normalTextureId").as_str(), 0);
    }

    #[cfg(feature = "3d")]
    pub(crate) unsafe fn uniform_light(&self, name: &str, value: &Light) {
        self.uniform_3fv((name.to_string() + ".position").as_str(), value.position);
        self.uniform_3fv((name.to_string() + ".direction").as_str(), value.direcetion);
        self.uniform_3fv((name.to_string() + ".color").as_str(), value.color.as_solid_vec());
        self.uniform_1f((name.to_string() + ".attenuation").as_str(), value.attenuation);
        self.uniform_1f((name.to_string() + ".cutoff").as_str(), value.cutoff);
        self.uniform_1f((name.to_string() + ".radius").as_str(), value.radius);
    }
}

pub struct OpenGLTexture {
    bytes: Option<Vec<u8>>,
    width: u32,
    height: u32,
    gl_id: u32,
}

impl OpenGLTexture {
    pub(crate) unsafe fn new(bytes: Vec<u8>) -> Self {
        OpenGLTexture {
            bytes: Some(bytes),
            width: 0,
            height: 0,
            gl_id: 0,
        }
    }

    pub(crate) unsafe fn make(&mut self) {
        if self.gl_id != 0 || self.bytes.is_none() {
            return;
        }
        let image = image::io::Reader::new(Cursor::new(self.bytes.take().unwrap())).with_guessed_format().unwrap();
        let image = image.decode().unwrap().into_rgba8();
        self.width = image.width();
        self.height = image.height();

        gl::GenTextures(1, &mut self.gl_id);
        gl::BindTexture(gl::TEXTURE_2D, self.gl_id);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA8 as GLint, self.width as GLsizei, self.height as GLsizei, 0, gl::RGBA, gl::UNSIGNED_BYTE, image.as_ptr() as *const c_void);
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }

    pub(crate) unsafe fn bind(&mut self, index: u8) {
        gl::ActiveTexture(gl::TEXTURE0 + index as u32);
        gl::BindTexture(gl::TEXTURE_2D, self.gl_id);
    }

    pub(crate) unsafe fn unbind(&mut self) {
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }

    pub(crate) fn get_width(&self) -> u32 {
        self.width
    }

    pub(crate) fn get_height(&self) -> u32 {
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
    vbo: u32,
    ibo: u32,
    camera: Option<Camera2D>,
}

impl OpenGLRenderProcessor2D {
    fn new() -> Self {
        OpenGLRenderProcessor2D {
            framebuffer: 0,
            width: 0,
            height: 0,
            vbo: 0,
            ibo: 0,
            camera: None,
        }
    }

    fn setup(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
        self.vbo = gen_buffer_id();
        self.ibo = gen_buffer_id();
    }

    fn resize(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    fn set_framebuffer(&mut self, framebuffer: u32) {
        self.framebuffer = framebuffer;
    }

    fn set_camera(&mut self, cam: Camera2D) {
        self.camera = Some(cam);
    }
}

macro_rules! vert_attrib {
    ($idx:expr, $size:ident, $off:ident) => {
        gl::VertexAttribPointer($idx, batch_layout_2d::$size as GLint, gl::FLOAT, 0, batch_layout_2d::VERTEX_SIZE_BYTES as GLsizei, batch_layout_2d::$off as *const _);
        gl::EnableVertexAttribArray($idx);
    };
}

impl RenderProcessor2D for OpenGLRenderProcessor2D {
    #[allow(clippy::too_many_arguments)]
    fn process_data(&self, tex: &mut [Option<Rc<RefCell<Texture>>>], tex_id: &[u32], indices: &[u32], vertices: &[f32], shader: &mut Shader, render_mode: u8) {
        let mut i: u8 = 0;
        for t in tex.iter_mut().flatten() {
            t.borrow_mut().bind(i);
            i += 1;
        }

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * 4) as GLsizeiptr, vertices.as_ptr() as *const c_void, gl::DYNAMIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (indices.len() * 4) as GLsizeiptr, indices.as_ptr() as *const c_void, gl::DYNAMIC_DRAW);

            if !tex.is_empty() {
                shader.uniform_iv("TEX_SAMPLER", tex_id.iter().map(|u| { *u as i32 }).collect::<Vec<_>>().as_slice());
            }

            shader.uniform_1i("uResX", self.width);
            shader.uniform_1i("uResY", self.height);
            shader.uniform_4fm("uProjection", self.camera.as_ref().unwrap().get_projection_mat());
            shader.uniform_4fm("uView", self.camera.as_ref().unwrap().get_view_mat());

            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);

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
}

#[cfg(feature = "3d")]
pub(crate) struct OpenGLRenderProcessor3D {
    framebuffer: u32,
    width: i32,
    height: i32,
    vbo: u32,
    ibo: u32,
    camera: Option<Camera3D>,
}

#[cfg(feature = "3d")]
impl OpenGLRenderProcessor3D {
    fn new() -> Self {
        OpenGLRenderProcessor3D {
            framebuffer: 0,
            width: 0,
            height: 0,
            vbo: 0,
            ibo: 0,
            camera: None,
        }
    }

    fn setup(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
        self.vbo = gen_buffer_id();
        self.ibo = gen_buffer_id();
    }

    fn resize(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    fn set_framebuffer(&mut self, framebuffer: u32) {
        self.framebuffer = framebuffer;
    }

    fn set_camera(&mut self, cam: Camera3D) {
        self.camera = Some(cam);
    }
}

#[cfg(feature = "3d")]
impl RenderProcessor3D for OpenGLRenderProcessor3D {
    fn process_batch(&self, tex: &mut [Option<Rc<RefCell<Texture>>>], tex_id: &[u32], indices: &[u32], vertices: &[f32], shader: &mut Shader, render_mode: u8) {

    }

    fn process_model(&self, tex: &mut [Option<Rc<RefCell<Texture>>>], tex_id: &[u32], indices: &[u32], vertices: &[f32], shader: &mut Shader, render_mode: u8) {

    }
}