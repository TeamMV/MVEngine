use alloc::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;
use std::io::Read;
use std::ops::DerefMut;
use std::sync::Arc;
use glam::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};
use glfw::ffi::{CLIENT_API, DECORATED, FALSE, glfwCreateWindow, glfwDefaultWindowHints, glfwDestroyWindow, glfwGetPrimaryMonitor, glfwGetVideoMode, glfwGetWindowPos, glfwMakeContextCurrent, glfwPollEvents, glfwSetWindowMonitor, glfwSetWindowShouldClose, glfwSetWindowSizeCallback, glfwShowWindow, glfwSwapBuffers, glfwSwapInterval, GLFWwindow, glfwWindowHint, glfwWindowShouldClose, NO_API, RESIZABLE, TRUE, VISIBLE};
use glsl_to_spirv::ShaderType;
use mvutils::utils::{AsCStr, TetrahedronOp, Time};
use once_cell::sync::Lazy;
use vulkano::device::Device;
use vulkano::shader::ShaderModule;
use crate::ApplicationInfo;
use crate::assets::{ReadableAssetManager, SemiAutomaticAssetManager};
use crate::render::camera::Camera;
use crate::render::draw::Draw2D;
use crate::render::{EFFECT_VERT, EMPTY_EFFECT_FRAG, glfwFreeCallbacks, load_render_assets, RenderCore};
use crate::render::shared::{ApplicationLoop, EffectShader, RenderProcessor2D, RunningWindow, Shader, ShaderPassInfo, Texture, Window, WindowCreateInfo};
use crate::render::vulkan::internal::Vulkan;

static mut VK_WINDOWS: Lazy<HashMap<*mut GLFWwindow, *mut VulkanWindow>> = Lazy::new(HashMap::new);

macro_rules! static_listener {
    ($name:ident, $inner:ident, $($params:ident: $types:ty),+) => {
        extern "C" fn $name(window: *mut GLFWwindow, $($params: $types),+) {
            unsafe {
                let window = VK_WINDOWS.get_mut(&window);
                if let Some(window) = window {
                    window.as_mut().unwrap().$inner($($params),+);
                }
            }
        }
    };
}

static_listener!(res, resize, w: i32, h: i32);

pub struct VulkanWindow {
    info: WindowCreateInfo,
    assets: Rc<RefCell<SemiAutomaticAssetManager>>,
    vulkan: Option<Vulkan>,
    app: *const ApplicationInfo,

    core: *mut RenderCore,
    window: *mut GLFWwindow,

    current_fps: u16,
    current_ups: u16,
    current_frame: u64,

    size_buf: [i32; 4],

    draw_2d: Option<Draw2D>,
    render_2d: VulkanRenderProcessor2D,
    shaders: HashMap<String, Rc<RefCell<EffectShader>>>,
    enabled_shaders: Vec<ShaderPassInfo>,
    //shader_pass: ,
    frame_buf: u32,
    texture_buf: u32,

    camera: Camera,

    res: (i32, i32)
}

impl VulkanWindow {
    pub(crate) unsafe fn new(info: WindowCreateInfo, assets: Rc<RefCell<SemiAutomaticAssetManager>>, core: *mut RenderCore, app: *const ApplicationInfo) -> Self {
        VulkanWindow {
            info,
            assets,
            vulkan: None,
            app,

            core,
            window: std::ptr::null_mut(),

            current_fps: 0,
            current_ups: 0,
            current_frame: 0,

            size_buf: [0; 4],
            render_2d: VulkanRenderProcessor2D::new(),
            shaders: HashMap::new(),
            enabled_shaders: Vec::with_capacity(10),
            //shader_pass: ,
            frame_buf: 0,
            texture_buf: 0,

            draw_2d: None,
            camera: Camera::new_2d(),
            res: (0, 0),
        }
    }

    fn init(&mut self) -> bool {
        unsafe {
            glfwDefaultWindowHints();
            glfwWindowHint(VISIBLE, FALSE);
            glfwWindowHint(CLIENT_API, NO_API);
            glfwWindowHint(DECORATED, self.info.decorated.yn(TRUE, FALSE));
            glfwWindowHint(RESIZABLE, self.info.resizable.yn(TRUE, FALSE));

            self.window = glfwCreateWindow(self.info.width, self.info.height, self.info.title.as_c_str().as_ptr(), std::ptr::null_mut(), std::ptr::null_mut());
            VK_WINDOWS.insert(self.window, self);

            let vulkan = Vulkan::init(self.app.as_ref().unwrap(), self.window, self.info.width as u32, self.info.height as u32);
            if vulkan.is_err() {
                return false;
            }
            self.vulkan = Some(vulkan.unwrap());

            glfwMakeContextCurrent(self.window);
            glfwSwapInterval(self.info.vsync.yn(1, 0));

            glfwShowWindow(self.window);

            //gl::Enable(gl::CULL_FACE);
            //gl::CullFace(gl::BACK);
            //gl::Enable(gl::BLEND);
            //gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            //gl::Enable(gl::DEPTH_TEST);
            //gl::DepthMask(gl::TRUE);
            //gl::DepthFunc(gl::LEQUAL);

            //self.shader_pass.set_ibo(self.render_2d.gen_buffer_id());

            glfwSetWindowSizeCallback(self.window, Some(res));
            true
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

                    application_loop.update(RunningWindow::Vulkan(self));
                    ticks += 1;
                    delta_u -= 1.0;
                }
                if delta_f >= 1.0 {
                    //gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
                    //gl::Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
                    //draws
                    self.draw_2d.as_mut().unwrap().reset_canvas();

                    application_loop.draw(RunningWindow::Vulkan(self));

                    let len = self.enabled_shaders.len();

                    if len > 0 {
                        //gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.frame_buf);
                        //gl::Clear(COLOR_BUFFER_BIT);
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
                            //self.shader_pass.render(shader.borrow_mut().deref_mut(), f_buf, self.texture_buf, self.current_frame as i32);
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
            VK_WINDOWS.remove(&self.window);
            glfwFreeCallbacks(self.window);
            glfwDestroyWindow(self.window);
        }
    }

    fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            self.info.width = width;
            self.info.height = height;
            self.vulkan.as_mut().unwrap().resize(width as u32, height as u32);
            //viewport(width, height);
            //self.gen_render_buffer();
            self.render_2d.resize(width, height);
            self.draw_2d.as_mut().unwrap().resize(width, height);
            //self.shader_pass.resize(width, height);
            self.camera.update_projection_mat(width, height);
        }
    }

    pub(crate) fn run(&mut self, application_loop: impl ApplicationLoop) {
        if !self.init() {
            unsafe {
                VK_WINDOWS.remove(&self.window);
                glfwDestroyWindow(self.window);
                self.core.as_mut().unwrap().rollback();
                let mut backup = self.core.as_mut().unwrap().create_window(self.info.clone());
                backup.run(application_loop);
                return;
            }
        }

        if self.info.fullscreen {
            self.set_fullscreen(true);
        }

        load_render_assets(self.assets.clone());

        //self.gen_render_buffer();
        //self.shader_pass.resize(self.info.width, self.info.height);
        self.render_2d.resize(self.info.width, self.info.height);
        self.render_2d.set_vulkan(self.vulkan.as_mut().unwrap());
        let shader = self.assets.borrow().get_shader("default");
        let font = self.assets.borrow().get_font("default");
        let mut binding = shader.borrow_mut();
        let vk_shader = binding.get_vk();
        //self.vulkan.as_mut().unwrap().set_shader_2d(vk_shader, self.info.width as u32, self.info.height as u32);
        self.draw_2d = Some(Draw2D::new(shader.clone(), font, self.info.width, self.info.height, self.res, self.get_dpi()));

        self.camera.update_projection_mat(self.info.width, self.info.height);
        application_loop.start(RunningWindow::Vulkan(self));

        self.running(&application_loop);
        application_loop.stop(RunningWindow::Vulkan(self));
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

    pub(crate) fn get_resolution(&self) -> (i32, i32) {
        self.res
    }

    pub(crate) fn get_dpi(&self) -> f32 {
        self.res.0 as f32 / self.res.1 as f32
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

    pub(crate) fn get_draw_2d(&mut self) -> &mut Draw2D {
        self.draw_2d.as_mut().expect("The Draw2D is not initialized yet!")
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

    pub(crate) fn get_camera(&self) -> &Camera {
        &self.camera
    }
}

pub struct VulkanShader {
    vertex_code: Option<String>,
    fragment_code: Option<String>,
    vertex: Option<Arc<ShaderModule>>,
    fragment: Option<Arc<ShaderModule>>,
}

impl VulkanShader {
    pub unsafe fn new(vertex: &str, fragment: &str) -> Self {
        VulkanShader {
            vertex_code: Some(vertex.to_string()),
            fragment_code: Some(fragment.to_string()),
            vertex: None,
            fragment: None,
        }
    }

    pub unsafe fn make(&mut self) {}

    pub unsafe fn vk_make(&mut self, device: Arc<Device>) {
        if self.vertex_code.is_none() || self.fragment_code.is_none() {
            return;
        }
        let vert = self.vertex_code.take().unwrap();
        let frag = self.fragment_code.take().unwrap();
        self.vertex = Some(ShaderModule::from_bytes(
            device.clone(),
            &glsl_to_spirv::compile(vert.as_str(), ShaderType::Vertex)
                .expect("Vertex shader failed to compile!")
                .bytes()
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
        ).expect("Vertex shader failed to compile!"));

        self.fragment = Some(ShaderModule::from_bytes(
            device,
            &glsl_to_spirv::compile(frag.as_str(), ShaderType::Fragment)
                .expect("Fragment shader failed to compile!")
                .bytes()
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
        ).expect("Fragment shader failed to compile!"));
    }

    pub unsafe fn vk_get(&self, t: u8) -> Arc<ShaderModule> {
        if t == 0 {
            return self.vertex.clone().unwrap();
        }
        return self.fragment.clone().unwrap();
    }

    pub unsafe fn bind(&mut self) {}

    pub unsafe fn uniform_1f(&self, name: &str, value: f32) {

    }

    pub unsafe fn uniform_1i(&self, name: &str, value: i32) {

    }

    pub unsafe fn uniform_fv(&self, name: &str, value: &[f32]) {

    }

    pub unsafe fn uniform_iv(&self, name: &str, value: &[i32]) {

    }

    pub unsafe fn uniform_2fv(&self, name: &str, value: Vec2) {

    }

    pub unsafe fn uniform_3fv(&self, name: &str, value: Vec3) {

    }

    pub unsafe fn uniform_4fv(&self, name: &str, value: Vec4) {

    }

    pub unsafe fn uniform_2fm(&self, name: &str, value: Mat2) {

    }

    pub unsafe fn uniform_3fm(&self, name: &str, value: Mat3) {

    }

    pub unsafe fn uniform_4fm(&self, name: &str, value: Mat4) {

    }
}

pub struct VulkanTexture {
    bytes: Option<Vec<u8>>,
    width: u32,
    height: u32,
}

impl VulkanTexture {
    pub unsafe fn new(bytes: Vec<u8>) -> Self {
        VulkanTexture {
            bytes: Some(bytes),
            width: 0,
            height: 0,
        }
    }

    pub unsafe fn make(&mut self) {

    }

    pub unsafe fn bind(&mut self, index: u8) {

    }

    pub unsafe fn unbind(&mut self) {

    }

    pub fn get_width(&self) -> u32 {
        0
    }

    pub fn get_height(&self) -> u32 {
        0
    }

    pub fn get_id(&self) -> u32 {
        0
    }
}

pub struct VulkanRenderProcessor2D {
    framebuffer: u32,
    width: i32,
    height: i32,
    camera: Option<Camera>,
    vulkan: *mut Vulkan
}

impl VulkanRenderProcessor2D {
    fn new() -> Self {
        VulkanRenderProcessor2D {
            framebuffer: 0,
            width: 0,
            height: 0,
            camera: None,
            vulkan: std::ptr::null_mut()
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

    fn set_vulkan(&mut self, vulkan: *mut Vulkan) {
        self.vulkan = vulkan;
    }
}

impl RenderProcessor2D for VulkanRenderProcessor2D {
    #[allow(clippy::too_many_arguments)]
    fn process_data(&self, tex: &mut [Option<Rc<RefCell<Texture>>>], tex_id: &[u32], indices: &[u32], vertices: &[f32], vbo: u32, ibo: u32, shader: &mut Shader, render_mode: u8) {
        unsafe {
            let vk = self.vulkan.as_mut().unwrap();

            let mut i: u8 = 0;
            for t in tex.iter_mut().flatten() {
                t.borrow_mut().bind(i);
                i += 1;
            }

            let ibo = vk.buffer_indices(indices);
            let vbo = vk.buffer_vertices(vertices);

            if !tex.is_empty() {
                shader.uniform_iv("TEX_SAMPLER", tex_id.iter().map(|u| { *u as i32 }).collect::<Vec<_>>().as_slice());
            }

            shader.uniform_1i("uResX", self.width);
            shader.uniform_1i("uResY", self.height);
            shader.uniform_4fm("uProjection", self.camera.as_ref().unwrap().get_projection_mat());
            shader.uniform_4fm("uView", self.camera.as_ref().unwrap().get_view_mat());

            //gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);

            let draw = vk.gen_command_buffer_2d(vbo, ibo, indices.len());
            vk.run(draw);
        }
    }

    fn gen_buffer_id(&self) -> u32 {
        0
    }

    fn adapt_render_mode(&self, render_mode: u8) -> u8 {
        render_mode
    }
}
