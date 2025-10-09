use crate::rendering::backend::buffer::Buffer;
use crate::rendering::backend::descriptor_set::DescriptorSet;
use crate::rendering::backend::device::Device;
use crate::rendering::backend::pipeline::Pipeline;

pub struct XRenderer3DPipelines {
    pub triangle: Pipeline,
    pub triangle_strip: Pipeline,
}

pub struct XRenderer3DBuffers {
    pub camera: Buffer,
    pub transform: Buffer,
}

pub struct XRenderer3DSets {
    pub camera: DescriptorSet,
    pub transform: DescriptorSet,
}

impl XRenderer3DPipelines {
    pub fn new(device: Device) -> Self {

    }
}