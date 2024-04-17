use crate::render::backend::buffer::MVBufferCreateInfo;
use crate::render::backend::vulkan::device::VkDevice;
use std::sync::Arc;

pub(crate) struct CreateInfo {
    pub instance_size: ash::vk::DeviceSize,
    pub instance_count: ash::vk::DeviceSize,
    pub usage_flags: ash::vk::BufferUsageFlags,
    pub memory_properties: ash::vk::MemoryPropertyFlags,
    pub minimum_alignment: ash::vk::DeviceSize,
    pub memory_usage_flags: gpu_alloc::UsageFlags,

    #[cfg(debug_assertions)]
    pub debug_name: std::ffi::CString,
}

impl From<MVBufferCreateInfo> for CreateInfo {
    fn from(value: MVBufferCreateInfo) -> Self {
        CreateInfo {
            instance_size: value.instance_size,
            instance_count: value.instance_count as u64,
            usage_flags: ash::vk::BufferUsageFlags::from_raw(value.buffer_usage.bits()),
            memory_properties: ash::vk::MemoryPropertyFlags::from_raw(
                value.memory_properties.bits() as u32,
            ),
            minimum_alignment: value.minimum_alignment,
            memory_usage_flags: value.memory_usage,

            #[cfg(debug_assertions)]
            debug_name: crate::render::backend::to_ascii_cstring(value.label.unwrap_or_default()),
        }
    }
}

pub struct VkBuffer {
    device: Arc<VkDevice>,

    handle: ash::vk::Buffer,
    mapped: *mut u8,
    block: Option<gpu_alloc::MemoryBlock<ash::vk::DeviceMemory>>,
    buffer_size: ash::vk::DeviceSize,
    instance_count: ash::vk::DeviceSize,
    instance_size: ash::vk::DeviceSize,
    alignment_size: ash::vk::DeviceSize,
    usage_flags: ash::vk::BufferUsageFlags,
    memory_properties: ash::vk::MemoryPropertyFlags,
    memory_usage_flags: gpu_alloc::UsageFlags,
}

impl VkBuffer {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: CreateInfo) -> Self {
        let alignment =
            Self::get_alignment(&create_info.instance_size, &create_info.minimum_alignment);
        let buffer_size = alignment * create_info.instance_count;

        let mut usage_flags = create_info.usage_flags;
        if create_info
            .memory_properties
            .contains(ash::vk::MemoryPropertyFlags::DEVICE_LOCAL)
        {
            usage_flags |= ash::vk::BufferUsageFlags::TRANSFER_DST;
        }

        let vk_create_info = ash::vk::BufferCreateInfo::builder()
            .size(buffer_size)
            .usage(usage_flags)
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);

        let (buffer, block) = device.allocate_buffer(
            &vk_create_info,
            create_info.memory_properties,
            gpu_alloc::UsageFlags::HOST_ACCESS,
        );

        #[cfg(debug_assertions)]
        device.set_object_name(
            &ash::vk::ObjectType::BUFFER,
            ash::vk::Handle::as_raw(buffer),
            create_info.debug_name.as_c_str(),
        );

        Self {
            device,
            handle: buffer,
            block: Some(block),
            mapped: std::ptr::null_mut(),
            buffer_size,
            instance_size: alignment,
            instance_count: create_info.instance_count,
            alignment_size: alignment,
            usage_flags,
            memory_properties: create_info.memory_properties,
            memory_usage_flags: create_info.memory_usage_flags,
        }
    }

    pub(crate) fn write_to_buffer(
        &mut self,
        data: &[u8],
        offset: ash::vk::DeviceSize,
        provided_cmd: Option<ash::vk::CommandBuffer>,
    ) {
        let ptr = data.as_ptr();
        let size = data.len();

        // Host Visible
        if !self
            .memory_properties
            .contains(ash::vk::MemoryPropertyFlags::DEVICE_LOCAL)
        {
            let need_to_map_buffer = self.mapped.is_null();
            if need_to_map_buffer {
                self.map();
            }

            unsafe { self.mapped = self.mapped.offset(offset as isize) };

            unsafe { std::ptr::copy_nonoverlapping(ptr, self.mapped, self.buffer_size as usize) };

            unsafe { self.mapped = self.mapped.offset(-(offset as isize)) };

            if need_to_map_buffer {
                self.unmap()
            }
        } else {
            // Device Local
            let buffer_create_info = CreateInfo {
                instance_size: size as ash::vk::DeviceSize,
                instance_count: 1,
                usage_flags: ash::vk::BufferUsageFlags::TRANSFER_SRC,
                memory_properties: ash::vk::MemoryPropertyFlags::HOST_VISIBLE,
                minimum_alignment: 1,
                memory_usage_flags: gpu_alloc::UsageFlags::HOST_ACCESS
                    | gpu_alloc::UsageFlags::TRANSIENT
                    | gpu_alloc::UsageFlags::UPLOAD,

                #[cfg(debug_assertions)]
                debug_name: std::ffi::CString::new("Staging Buffer").unwrap(),
            };

            let mut staging_buffer = Self::new(self.device.clone(), buffer_create_info);

            staging_buffer.map();

            staging_buffer.write_to_buffer(data, 0, None);

            staging_buffer.unmap();

            Self::copy_buffer(
                &staging_buffer,
                self,
                size as ash::vk::DeviceSize,
                0,
                offset,
                provided_cmd,
            );
        }
    }

    pub(crate) fn get_descriptor_info(
        &self,
        size: ash::vk::DeviceSize,
        offset: ash::vk::DeviceSize,
    ) -> ash::vk::DescriptorBufferInfo {
        ash::vk::DescriptorBufferInfo {
            buffer: self.handle,
            offset,
            range: size,
        }
    }

    pub(crate) fn copy_buffer(
        src_buffer: &VkBuffer,
        dst_buffer: &mut VkBuffer,
        size: ash::vk::DeviceSize,
        src_offset: ash::vk::DeviceSize,
        dst_offset: ash::vk::DeviceSize,
        provided_cmd: Option<ash::vk::CommandBuffer>,
    ) {
        let (cmd, end) = if let Some(cmd) = provided_cmd {
            (cmd, false)
        } else {
            (
                src_buffer
                    .device
                    .begin_single_time_command(src_buffer.device.get_graphics_command_pool()),
                true,
            )
        };

        let copy_region = [ash::vk::BufferCopy {
            src_offset,
            dst_offset,
            size,
        }];

        unsafe {
            src_buffer.device.get_device().cmd_copy_buffer(
                cmd,
                src_buffer.get_buffer(),
                dst_buffer.get_buffer(),
                &copy_region,
            )
        }

        if end {
            src_buffer.device.end_single_time_command(
                cmd,
                src_buffer.device.get_graphics_command_pool(),
                src_buffer.device.get_graphics_queue(),
            );
        }
    }

    pub(crate) fn map(&mut self) {
        if self
            .memory_properties
            .contains(ash::vk::MemoryPropertyFlags::DEVICE_LOCAL)
        {
            log::error!("Can't map device local buffer!");
            panic!();
        }

        let block = self
            .block
            .as_mut()
            .expect("Memory block of buffer should never be None");

        self.mapped = unsafe {
            block.map(
                gpu_alloc_ash::AshMemoryDevice::wrap(self.device.get_device()),
                0,
                block.size() as usize,
            )
        }
        .unwrap_or_else(|e| {
            log::error!("Failed to flush buffer, error: {e}");
            panic!()
        })
        .as_ptr();
    }

    pub(crate) fn unmap(&mut self) {
        if self
            .memory_properties
            .contains(ash::vk::MemoryPropertyFlags::DEVICE_LOCAL)
        {
            log::error!("Can't unmap device local buffer!");
            panic!();
        }

        unsafe {
            self.block
                .as_mut()
                .expect("Memory block of buffer should never be None")
                .unmap(gpu_alloc_ash::AshMemoryDevice::wrap(
                    self.device.get_device(),
                ))
        };
        self.mapped = std::ptr::null_mut();
    }

    fn get_alignment(
        instance_size: &ash::vk::DeviceSize,
        min_offset_alignment: &ash::vk::DeviceSize,
    ) -> ash::vk::DeviceSize {
        (instance_size + min_offset_alignment - 1) & !(min_offset_alignment - 1)
    }

    pub(crate) fn get_buffer(&self) -> ash::vk::Buffer {
        self.handle
    }

    pub(crate) fn get_size(&self) -> ash::vk::DeviceSize {
        self.buffer_size
    }

    pub(crate) fn get_usage_flags(&self) -> ash::vk::BufferUsageFlags {
        self.usage_flags
    }

    pub(crate) fn get_memory_properties(&self) -> ash::vk::MemoryPropertyFlags {
        self.memory_properties
    }

    pub(crate) fn is_mapped(&self) -> bool {
        !self.mapped.is_null()
    }
}

impl Drop for VkBuffer {
    fn drop(&mut self) {
        if !self.mapped.is_null() {
            self.unmap();
        }

        self.device.deallocate_buffer(
            self.handle,
            self.block
                .take()
                .expect("Memory block of buffer should never be None"),
        );
    }
}
