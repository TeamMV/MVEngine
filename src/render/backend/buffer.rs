use crate::render::backend::command_buffer::CommandBuffer;
use crate::render::backend::device::Device;
use crate::render::backend::vulkan::buffer::VkBuffer;
use bitflags::bitflags;
use mvcore_proc_macro::graphics_item;

pub struct MVBufferCreateInfo {
    pub instance_size: u64,
    pub instance_count: u32,
    pub buffer_usage: BufferUsage,
    pub memory_properties: MemoryProperties,
    pub minimum_alignment: u64,
    pub memory_usage: gpu_alloc::UsageFlags,

    pub label: Option<String>,
}

#[graphics_item(ref)]
pub enum Buffer {
    Vulkan(VkBuffer),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

impl Buffer {
    pub fn new(device: Device, create_info: MVBufferCreateInfo) -> Self {
        match device {
            Device::Vulkan(device) => Buffer::Vulkan(VkBuffer::new(device, create_info.into())),
            #[cfg(target_os = "macos")]
            Device::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Device::DirectX => unimplemented!(),
        }
    }

    pub fn write(&mut self, data: &[u8], offset: u64, command_buffer: Option<&CommandBuffer>) {
        match self {
            Buffer::Vulkan(buffer) => buffer.write_to_buffer(
                data,
                offset,
                command_buffer.map(|buffer| buffer.as_vulkan().get_handle()),
            ),
            #[cfg(target_os = "macos")]
            Buffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Buffer::DirectX => unimplemented!(),
        }
    }

    pub fn get_size(&self) -> u64 {
        match self {
            Buffer::Vulkan(buffer) => buffer.get_size(),
            #[cfg(target_os = "macos")]
            Buffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Buffer::DirectX => unimplemented!(),
        }
    }

    pub fn get_descriptor_info(&self, size: u64, offset: u64) -> DescriptorBufferInfo {
        match self {
            Buffer::Vulkan(buffer) => {
                DescriptorBufferInfo::Vulkan(buffer.get_descriptor_info(size, offset))
            }
            #[cfg(target_os = "macos")]
            Buffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Buffer::DirectX => unimplemented!(),
        }
    }

    pub fn copy_buffer(
        src: &mut Buffer,
        dst: &mut Buffer,
        size: u64,
        src_offset: u64,
        dst_offset: u64,
        command_buffer: Option<&CommandBuffer>,
    ) {
        match (src, dst) {
            (Buffer::Vulkan(src), Buffer::Vulkan(dst)) => VkBuffer::copy_buffer(
                src,
                dst,
                size,
                src_offset,
                dst_offset,
                command_buffer.map(|buffer| buffer.as_vulkan().get_handle()),
            ),
            #[cfg(target_os = "macos")]
            (Buffer::Metal, Buffer::Metal) => unimplemented!(),
            #[cfg(target_os = "windows")]
            (Buffer::DirectX, Buffer::DirectX) => unimplemented!(),
            #[cfg(any(target_os = "windows", target_os = "macos"))]
            (_, _) => unreachable!(),
        }
    }

    pub fn map(&mut self) {
        match self {
            Buffer::Vulkan(buffer) => buffer.map(),
            #[cfg(target_os = "macos")]
            Buffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Buffer::DirectX => unimplemented!(),
        }
    }

    pub fn unmap(&mut self) {
        match self {
            Buffer::Vulkan(buffer) => buffer.unmap(),
            #[cfg(target_os = "macos")]
            Buffer::Metal => unimplemented!(),
            #[cfg(target_os = "windows")]
            Buffer::DirectX => unimplemented!(),
        }
    }
}

pub enum DescriptorBufferInfo {
    Vulkan(ash::vk::DescriptorBufferInfo),
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(target_os = "windows")]
    DirectX,
}

bitflags! {
    pub struct BufferUsage: u32 {
        const TRANSFER_SRC = 1;
        const TRANSFER_DST = 1 << 1;
        const UNIFORM_TEXEL_BUFFER = 1 << 2;
        const STORAGE_TEXEL_BUFFER = 1 << 3;
        const UNIFORM_BUFFER = 1 << 4;
        const STORAGE_BUFFER = 1 << 5;
        const INDEX_BUFFER = 1 << 6;
        const VERTEX_BUFFER = 1 << 7;
        const INDIRECT_BUFFER = 1 << 8;
        #[cfg(feature = "ray-tracing")]
        const SHADER_BINDING_TABLE_KHR = 1 << 10;
        const SHADER_DEVICE_ADDRESS = 1 << 17;
        #[cfg(feature = "ray-tracing")]
        const VACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR = 1 << 19;
        #[cfg(feature = "ray-tracing")]
        const ACCELERATION_STRUCTURE_STORAGE_KHR = 1 << 20;
    }
}

bitflags! {
    pub struct MemoryProperties: u8 {
        const DEVICE_LOCAL = 1;
        const HOST_VISIBLE = 1 << 1;
        const HOST_COHERENT = 1 << 2;
        const HOST_CACHED = 1 << 3;
        const LAZILY_ALLOCATED = 1 << 4;
        const PROTECTED = 1 << 5;
    }
}
