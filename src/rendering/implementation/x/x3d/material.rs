use std::mem;
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::sync::Arc;
use gpu_alloc::UsageFlags;
use indexmap::IndexSet;
use itertools::Itertools;
use mvutils::once::CreateOnce;
use mvutils::utils::Time;
use crate::rendering::backend::buffer::{Buffer, BufferUsage, MVBufferCreateInfo, MemoryProperties};
use crate::rendering::backend::command_buffer::{CommandBuffer, CommandBufferLevel, MVCommandBufferCreateInfo};
use crate::rendering::backend::device::{CommandPool, Device};
use crate::rendering::backend::swapchain::Swapchain;
use crate::rendering::implementation::model::material::Material;
use crate::rendering::implementation::model::Model;
use crate::rendering::implementation::x::x3d::model::XLoadedModel;

pub struct XMaterials {
    version: u64,
    materials: IndexSet<Material>,
    buffer: CreateOnce<Buffer>,
}

impl XMaterials {
    pub fn new() -> Self {
        Self {
            version: u64::time_millis(),
            materials: IndexSet::new(),
            buffer: CreateOnce::new(),
        }
    }

    pub fn on_model_load(&mut self, model: &Model) -> XLoadedModel {
        let mut indices = vec![];
        let mut index = self.materials.len();
        for mat in &model.materials {
            if self.materials.insert(mat.clone()) {
                index += 1;
                indices.push(index as u32);
            } else {
                if let Some(idx) = self.materials.get_index_of(mat) {
                    indices.push(idx as u32);
                }
            }
        }
        XLoadedModel {
            x_materials_version: self.version,
            used_materials: indices,
            mesh: model.mesh.clone(),
        }
    }

    pub fn on_models_loaded(&mut self, device: Device, swapchain: &mut Swapchain) {
        let buffer_ci = MVBufferCreateInfo {
            instance_size: 0,
            instance_count: 0,
            //fill this buffer with a staging buffer and vkCmdBufferCopy or however its called
            buffer_usage: BufferUsage::UNIFORM_BUFFER | BufferUsage::TRANSFER_DST,
            //i will order the vram portion today
            memory_properties: MemoryProperties::DEVICE_LOCAL,
            minimum_alignment: 0,
            //sounds good ill take it
            memory_usage: UsageFlags::FAST_DEVICE_ACCESS,
            label: Some(format!("XMaterials UBO {}", self.version)),
        };
        self.buffer.create(|| Buffer::new(device.clone(), buffer_ci));
        //force creation now and not during render lol
        let _ = self.buffer.deref();

        let staging_ci = MVBufferCreateInfo {
            instance_size: 0,
            instance_count: 0,
            buffer_usage: BufferUsage::TRANSFER_SRC | BufferUsage::UNIFORM_BUFFER,
            //cpu memory please
            memory_properties: MemoryProperties::HOST_VISIBLE,
            minimum_alignment: 0,
            //cpu again
            memory_usage: UsageFlags::HOST_ACCESS,
            label: Some(format!("XMaterials Staging Buffer {}", self.version)),
        };

        let mut staging_buffer = Buffer::new(device.clone(), staging_ci);

        //yes the self.materials is just temporary for creation
        let material_data = self.materials.drain(..).collect_vec();
        let data_size = material_data.len() * size_of::<Material>();
        //i think this works
        let data = unsafe { &*slice_from_raw_parts(material_data.as_ptr() as *mut u8, data_size) };

        let cmd_ci = MVCommandBufferCreateInfo {
            level: CommandBufferLevel::Primary,
            //idk prolly good enough
            pool: device.get_compute_command_pool(),
            label: Some(format!("XMaterials Staging Command Buffer {}", self.version)),
        };

        let cmd = CommandBuffer::new(device.clone(), cmd_ci);
        cmd.begin();
        cmd.write_buffer(&mut staging_buffer, data, 0);
        cmd.copy_buffers(&staging_buffer, &mut *self.buffer, data_size as u64, 0, 0);
        cmd.end();

        if let Err(e) = swapchain.submit_command_buffer(&cmd, 0) {
            log::error!("Error when submitting staging buffer! {e:?}");
            log::error!("Please press win+ctrl+shift+alt+f12+tab+del+ins to recreate swapchain!")
            //we cannot do shit here sorry swapchain :(
        }

        //important to drop staging buffer first bi think, because the material_data vec if being referenced
        //i regret saying this if write_buffer copies the data and does not hold a reference for 'buffer
        drop(staging_buffer);
    }

    pub fn clear(&mut self) {
        //TODO destroy the underlying buffer manually instead of relying on drop()?
        self.buffer = CreateOnce::new();
        self.version = u64::time_millis();
    }
}