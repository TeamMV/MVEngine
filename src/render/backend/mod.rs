pub(crate) mod buffer;
pub(crate) mod descriptor_set;
pub(crate) mod device;
pub(crate) mod framebuffer;
pub(crate) mod image;
pub(crate) mod pipeline;
pub(crate) mod push_constant;
pub(crate) mod sampler;
pub(crate) mod sbt;
pub(crate) mod shader;
pub(crate) mod swapchain;
pub(crate) mod vulkan;

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
