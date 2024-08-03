use crate::render::backend::buffer::{Buffer, BufferUsage, MemoryProperties, MVBufferCreateInfo};
use crate::render::backend::command_buffer::CommandBuffer;
use crate::render::backend::device::Device;

pub struct Mesh {
    device: Device,
    vertex_buffer: Buffer,
    vertex_count: u32,
    index_count: u32,
    index_buffer: Option<Buffer>,
    name: Option<String>,
}

impl Mesh {
    pub fn new(
        device: Device,
        vertices: &[u8],
        vertex_count: u32,
        indices: Option<&[u32]>,
        name: Option<String>,
    ) -> Mesh {
        let vertex_buffer = Self::create_vertex_buffer(device.clone(), vertices, name.clone());
        let len = indices
            .as_ref()
            .map(|indices| indices.len())
            .unwrap_or_default();
        let index_buffer =
            indices.map(|indices| Self::create_index_buffer(device.clone(), indices, name.clone()));

        Self {
            device,
            vertex_buffer,
            vertex_count,
            index_count: len as u32,
            index_buffer,
            name,
        }
    }

    fn create_vertex_buffer(device: Device, vertices: &[u8], name: Option<String>) -> Buffer {
        let mut vertex_buffer = Buffer::new(
            device.clone(),
            MVBufferCreateInfo {
                instance_size: 1,
                instance_count: vertices.len() as u32,
                buffer_usage: BufferUsage::VERTEX_BUFFER,
                memory_properties: MemoryProperties::DEVICE_LOCAL,
                minimum_alignment: 1,
                memory_usage: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
                label: name,
            },
        );

        vertex_buffer.write(vertices, 0, None);

        vertex_buffer
    }

    fn create_index_buffer(device: Device, indices: &[u32], name: Option<String>) -> Buffer {
        let mut index_buffer = Buffer::new(
            device.clone(),
            MVBufferCreateInfo {
                instance_size: 4,
                instance_count: indices.len() as u32,
                buffer_usage: BufferUsage::INDEX_BUFFER,
                memory_properties: MemoryProperties::DEVICE_LOCAL,
                minimum_alignment: 1,
                memory_usage: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
                label: name,
            },
        );

        let data =
            unsafe { std::slice::from_raw_parts(indices.as_ptr() as *const u8, indices.len() * 4) };
        index_buffer.write(data, 0, None);

        index_buffer
    }

    pub fn update_vertex_buffer(&mut self, vertices: &[u8]) {
        if vertices.len() > self.vertex_count as usize {
            self.vertex_buffer =
                Self::create_vertex_buffer(self.device.clone(), vertices, self.name.clone());
            self.vertex_count = vertices.len() as u32;
            return;
        }

        self.vertex_count = vertices.len() as u32;
        self.vertex_buffer.write(vertices, 0, None);
    }

    pub fn update_index_buffer(&mut self, indices: &[u32]) {
        if indices.len() <= self.index_count as usize {
            if let Some(buffer) = self.index_buffer.as_mut() {
                let data = unsafe {
                    std::slice::from_raw_parts(indices.as_ptr() as *const u8, indices.len() * 4)
                };
                buffer.write(data, 0, None);
                self.index_count = indices.len() as u32;
                return;
            }
        }
        self.index_count = indices.len() as u32;
        self.index_buffer = Some(Self::create_index_buffer(
            self.device.clone(),
            indices,
            self.name.clone(),
        ));
    }

    pub fn remove_index_buffer(&mut self) {
        self.index_buffer = None;
        self.index_count = 0;
    }

    pub fn draw(&self, cmd: &CommandBuffer) {
        cmd.bind_vertex_buffer(&self.vertex_buffer);

        if let Some(index_buffer) = &self.index_buffer {
            cmd.bind_index_buffer(index_buffer);

            cmd.draw_indexed(self.index_count, 0);
        } else {
            cmd.draw(self.vertex_count, 0);
        }
    }

    pub fn draw_instanced(&self, cmd: &CommandBuffer, first_instance: u32, instance_count: u32) {
        cmd.bind_vertex_buffer(&self.vertex_buffer);

        if let Some(index_buffer) = &self.index_buffer {
            cmd.bind_index_buffer(index_buffer);

            cmd.draw_indexed_instanced(self.index_count, instance_count, 0, first_instance);
        } else {
            cmd.draw_instanced(self.vertex_count, instance_count, 0, first_instance);
        }
    }
}
