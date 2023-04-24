use alloc::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use glam::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};
use glfw::ffi::GLFWwindow;
use once_cell::sync::Lazy;
use crate::assets::SemiAutomaticAssetManager;
use crate::render::camera::Camera;
use crate::render::draw::Draw2D;
use crate::render::shared::{ApplicationLoop, EffectShader, ShaderPassInfo, Window, WindowCreateInfo};

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

}

impl VulkanWindow {
    pub(crate) fn new(info: WindowCreateInfo, assets: Rc<RefCell<SemiAutomaticAssetManager>>) -> Self {
        todo!()
    }

    fn resize(&mut self, width: i32, height: i32) {
        
    }

    pub(crate) fn run(&mut self, application_loop: impl ApplicationLoop) {
        todo!()
    }

    pub(crate) fn stop(&mut self) {
        todo!()
    }

    pub(crate) fn get_width(&self) -> i32 {
        todo!()
    }

    pub(crate) fn get_height(&self) -> i32 {
        todo!()
    }

    pub(crate) fn get_resolution(&self) -> (i32, i32) {
        todo!()
    }

    pub(crate) fn get_dpi(&self) -> f32 {
        todo!()
    }

    pub(crate) fn get_fps(&self) -> u16 {
        todo!()
    }

    pub(crate) fn get_ups(&self) -> u16 {
        todo!()
    }

    pub(crate) fn get_frame(&self) -> u64 {
        todo!()
    }

    pub(crate) fn get_draw_2d(&mut self) -> &mut Draw2D {
        todo!()
    }

    pub(crate) fn set_fullscreen(&mut self, fullscreen: bool) {
        todo!()
    }

    fn get_glfw_window(&self) -> *mut GLFWwindow {
        todo!()
    }

    pub(crate) fn add_shader(&mut self, id: &str, shader: Rc<RefCell<EffectShader>>) {
        todo!()
    }

    pub(crate) fn queue_shader_pass(&mut self, info: ShaderPassInfo) {
        todo!()
    }

    pub(crate) fn get_camera(&self) -> &Camera {
        todo!()
    }
}

pub struct VulkanShader {

}

impl VulkanShader {
    pub unsafe fn new(vertex: &str, fragment: &str) -> Self {
        todo!()
    }

    pub unsafe fn make(&mut self) {

    }

    pub unsafe fn bind(&mut self) {

    }

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

}

impl VulkanTexture {
    pub unsafe fn new(bytes: Vec<u8>) -> Self {
        todo!()
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

}

impl VulkanRenderProcessor2D {

}
