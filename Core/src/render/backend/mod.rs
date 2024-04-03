use std::ffi::CString;

pub mod buffer;
pub mod command_buffer;
pub mod descriptor_set;
pub mod device;
pub mod framebuffer;
pub mod image;
pub mod pipeline;
pub mod push_constant;
pub mod sampler;
pub mod shader;
pub mod swapchain;
pub(crate) mod vulkan;

#[cfg(feature = "ray-tracing")]
pub mod sbt;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub enum Backend {
    #[default]
    Vulkan,
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct Extent2D {
    pub width: u32,
    pub height: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct Extent3D {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

pub(crate) fn to_ascii_cstring(input: String) -> CString {
    let ascii = input.chars().filter(|c| c.is_ascii()).collect::<String>();
    CString::new(ascii.as_bytes()).expect("CString::new failed")
}
