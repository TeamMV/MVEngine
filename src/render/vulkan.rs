use std::collections::HashMap;
use glfw::ffi::GLFWwindow;
use once_cell::sync::Lazy;

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
    fn resize(&mut self, width: i32, height: i32) {
        
    }
}

pub struct VulkanShader {

}

impl VulkanShader {

}

pub struct VulkanTexture {

}

impl VulkanTexture {

}

pub struct VulkanRenderProcessor2D {

}
