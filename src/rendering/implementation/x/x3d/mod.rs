use crate::rendering::backend::buffer::Buffer;
use crate::rendering::backend::descriptor_set::{DescriptorPool, DescriptorSet};
use crate::rendering::backend::device::Device;
use crate::rendering::backend::pipeline::{MVGraphicsPipelineCreateInfo, Pipeline};
use crate::rendering::implementation::model::Model;
use material::XMaterials;
use crate::rendering::implementation::x::core::XRendererCore;
use crate::rendering::implementation::x::x3d::model::XLoadedModel;
use crate::rendering::implementation::x::x3d::types::{
    XRenderer3DBuffers, XRenderer3DPipelines, XRenderer3DSets,
};

pub mod batch;
pub mod model;
pub mod types;
pub mod material;

pub struct XRenderer3DAddon {
    //vulkan
    device: Device,
    descriptor_pool: DescriptorPool,
    sets: Vec<XRenderer3DSets>,
    buffers: Vec<XRenderer3DBuffers>,
    pipelines: XRenderer3DPipelines,

    //general 3d
    materials: XMaterials,
    models: Vec<XLoadedModel>
}

impl XRenderer3DAddon {
    pub fn new(device: Device) -> Self {
        todo!()
    }

    pub fn upload_model(&mut self, model: &Model) {
        let m = self.materials.on_model_load(model);
        self.models.push(m);
    }
    
    //yoski i just realized that we will need `this` to call these methods taking in a core. we have to come up with something better bro
    pub fn finish_scene(&mut self, core: &mut XRendererCore) {
        self.materials.on_models_loaded(self.device.clone(), core.get_swapchain_mut());
    }
}
