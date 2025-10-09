use crate::rendering::backend::buffer::Buffer;
use crate::rendering::backend::descriptor_set::{DescriptorPool, DescriptorSet};
use crate::rendering::backend::device::Device;
use crate::rendering::backend::pipeline::{MVGraphicsPipelineCreateInfo, Pipeline};
use crate::rendering::implementation::model::Model;
use crate::rendering::implementation::x::x3d::collections::{XRenderer3DBuffers, XRenderer3DPipelines, XRenderer3DSets};

pub mod batch;
mod collections;

pub struct XRenderer3DAddon {
    device: Device,
    descriptor_pool: DescriptorPool,
    sets: Vec<XRenderer3DSets>,
    buffers: Vec<XRenderer3DBuffers>,
    pipelines: XRenderer3DPipelines,
}

impl XRenderer3DAddon {
    pub fn new(device: Device) -> Self {
        Self {}
    }

    pub fn draw_single_model(&mut self, model: Model) {

    }
}