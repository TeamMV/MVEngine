use std::ffi::CString;
use std::sync::Arc;
use ash::vk::Handle;
use crate::render::backend::buffer::MVBufferCreateInfo;
use crate::render::backend::to_ascii_cstring;
use crate::render::backend::vulkan::device::VkDevice;

pub(crate) struct CreateInfo {
    instance_size: ash::vk::DeviceSize,
    instance_count: ash::vk::DeviceSize,
    usage_flags: ash::vk::BufferUsageFlags,
    memory_properties: ash::vk::MemoryPropertyFlags,
    minimum_alignment: ash::vk::DeviceSize,
    no_pool: bool,

    #[cfg(debug_assertions)]
    debug_name: CString
}

impl From<MVBufferCreateInfo> for CreateInfo {
    fn from(value: MVBufferCreateInfo) -> Self {
        CreateInfo {
            instance_size: value.instance_size,
            instance_count: value.instance_count as u64,
            usage_flags: ash::vk::BufferUsageFlags::from_raw(value.usage.bits()),
            memory_properties: ash::vk::MemoryPropertyFlags::from_raw(value.memory_properties.bits() as u32),
            minimum_alignment: value.minimum_alignment,
            no_pool: value.no_pool,

            #[cfg(debug_assertions)]
            debug_name: to_ascii_cstring(value.label.unwrap_or("".to_string())),
        }
    }
}

pub(crate) struct VkBuffer {
    device: Arc<VkDevice>,

    handle: ash::vk::Buffer,
    mapped: *mut u8,
    allocation: vk_mem::Allocation,
    buffer_size: ash::vk::DeviceSize,
    instance_count: ash::vk::DeviceSize,
    instance_size: ash::vk::DeviceSize,
    alignment_size: ash::vk::DeviceSize,
    usage_flags: ash::vk::BufferUsageFlags,
    memory_properties: ash::vk::MemoryPropertyFlags,
    no_pool: bool,
    pool: Option<vk_mem::AllocatorPool>

}

impl VkBuffer {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: CreateInfo) -> Self {
        let alignment = Self::get_alignment(&create_info.instance_size, &create_info.minimum_alignment);
        let buffer_size = alignment * create_info.instance_count;

        let vk_create_info = ash::vk::BufferCreateInfo::builder()
            .size(buffer_size)
            .usage(create_info.usage_flags)
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
            .build();

        let (buffer, allocation) = device.allocate_buffer(&vk_create_info, create_info.memory_properties, false);

        #[cfg(debug_assertions)]
        device.set_object_name(&ash::vk::ObjectType::BUFFER, buffer.as_raw(), create_info.debug_name.as_c_str());

        return Self {
            device,
            handle: buffer,
            allocation,
            mapped: std::ptr::null_mut(),
            buffer_size,
            instance_size: alignment,
            instance_count: create_info.instance_count,
            alignment_size: alignment,
            usage_flags: create_info.usage_flags,
            memory_properties: create_info.memory_properties,
            no_pool: false, // always false for now
            pool: None // always false for now
        }
    }

    // We'll need wrapper for these
    pub(crate) fn write_to_buffer(&mut self, data: &[u8], offset: ash::vk::DeviceSize, provided_cmd: Option<ash::vk::CommandBuffer>) {
        let ptr = data.as_ptr();
        let size = data.len();

        let mut cmd;
        let has_cmd_provided = provided_cmd.is_some();
        if !has_cmd_provided {
            cmd = self.device.begin_single_time_command(self.device.get_compute_command_pool());
        }
        else {
            cmd = provided_cmd.unwrap();
        }

        // Host Visible
        if !self.memory_properties.contains(ash::vk::MemoryPropertyFlags::DEVICE_LOCAL) {
            let need_to_map_buffer = self.mapped == std::ptr::null_mut();
            if need_to_map_buffer {
                self.map();
            }

            unsafe { std::ptr::copy_nonoverlapping(ptr, self.mapped, self.buffer_size as usize) };

            if need_to_map_buffer {
                self.unmap()
            }
        } else { // Device Local
            let buffer_create_info = CreateInfo {
                instance_size: size as ash::vk::DeviceSize,
                instance_count: self.instance_count,
                usage_flags: ash::vk::BufferUsageFlags::TRANSFER_SRC,
                memory_properties: ash::vk::MemoryPropertyFlags::HOST_VISIBLE,
                minimum_alignment: 1,
                no_pool: false,

                #[cfg(debug_assertions)]
                debug_name: CString::new("Staging Buffer").unwrap(),
            };

            let mut staging_buffer = Self::new(self.device.clone(), buffer_create_info);

            staging_buffer.map();

            staging_buffer.write_to_buffer(data, size as ash::vk::DeviceSize, Some(cmd));

            staging_buffer.unmap();

            Self::copy_buffer(&staging_buffer, self, size as ash::vk::DeviceSize, 0, 0, Some(cmd));
        }

        if !has_cmd_provided {
            self.device.end_single_time_command(cmd, self.device.get_compute_command_pool(), self.device.get_compute_queue());
        }
    }

    pub(crate) fn flush(&self) {
        unsafe { self.device.get_allocator().flush_allocation(*self.allocation, 0, *self.buffer_size) }.unwrap();
    }

    pub(crate) fn get_descriptor_info(&self, size: ash::vk::DeviceSize, offset: ash::vk::DeviceSize) -> ash::vk::DescriptorBufferInfo {
        ash::vk::DescriptorBufferInfo::builder()
            .buffer(self.handle)
            .offset(offset)
            .range(size)
            .build()
    }

    pub(crate) fn copy_buffer(src_buffer: &VkBuffer, dst_buffer: &VkBuffer, size: ash::vk::DeviceSize, src_offset: ash::vk::DeviceSize, dst_offset: ash::vk::DeviceSize, provided_cmd: Option<ash::vk::CommandBuffer>) {
        let (cmd, end) = if let Some(cmd) = provided_cmd {
            (cmd, false)
        } else {
            (src_buffer.device.begin_single_time_command(src_buffer.device.get_compute_command_pool()), true)
        };

        let copy_region = [ash::vk::BufferCopy::builder()
            .src_offset(src_offset)
            .dst_offset(dst_offset)
            .size(size)
            .build()];

        unsafe { src_buffer.device.get_device().cmd_copy_buffer(cmd, *src_buffer.get_buffer(), *dst_buffer.get_buffer(), &copy_region) }

        if end {
            src_buffer.device.end_single_time_command(cmd, src_buffer.device.get_compute_command_pool(), src_buffer.device.get_compute_queue());
        }
    }

    pub(crate) fn map(&mut self) {
        if self.memory_properties.contains(ash::vk::MemoryPropertyFlags::DEVICE_LOCAL) {
            log::error!("Can't map device local buffer!");
            panic!();
        }

        self.mapped = unsafe { self.device.get_allocator().map_memory(&mut self.allocation) }.unwrap_or_else(|e| {
            log::error!("Failed to map memory, error: {e}");
            panic!();
        });
    }

    pub(crate) fn unmap(&mut self) {
        if self.memory_properties.contains(ash::vk::MemoryPropertyFlags::DEVICE_LOCAL) {
            log::error!("Can't unmap device local buffer!");
            panic!();
        }

        unsafe { self.device.get_allocator().unmap_memory(&mut self.allocation) };
        self.mapped = std::ptr::null_mut();
    }

    fn get_alignment(instance_size: &ash::vk::DeviceSize, min_offset_alignment: &ash::vk::DeviceSize) -> ash::vk::DeviceSize {
        return (instance_size + min_offset_alignment - 1) & !(min_offset_alignment - 1);
    }

    pub(crate) fn get_buffer(&self) -> ash::vk::Buffer {
        self.handle
    }
}