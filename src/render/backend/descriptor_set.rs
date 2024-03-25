use mvcore_proc_macro::graphics_item;

#[graphics_item(copy)]
pub(crate) enum DescriptorSetLayout {
    Vulkan(ash::vk::DescriptorSetLayout),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}
