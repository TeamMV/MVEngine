use std::ffi::CString;

pub(crate) mod buffer;
pub(crate) mod command_buffer;
pub(crate) mod descriptor_set;
pub(crate) mod device;
pub(crate) mod framebuffer;
pub(crate) mod image;
pub(crate) mod pipeline;
pub(crate) mod push_constant;
pub(crate) mod sampler;
pub(crate) mod shader;
pub(crate) mod swapchain;
pub(crate) mod vulkan;

#[cfg(feature = "ray-tracing")]
pub(crate) mod sbt;

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

pub(crate) fn to_ascii_cstring(input: String) -> CString {
    let ascii = input.chars().filter(|c| c.is_ascii()).collect::<String>();
    CString::new(ascii.as_bytes()).expect("CString::new failed")
}
