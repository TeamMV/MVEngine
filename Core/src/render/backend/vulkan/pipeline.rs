use crate::render::backend::pipeline::{
    AttributeType, Compute, CullMode, Graphics, MVComputePipelineCreateInfo,
    MVGraphicsPipelineCreateInfo, PipelineType, PushConstant, Topology,
};
#[cfg(feature = "ray-tracing")]
use crate::render::backend::pipeline::{MVRayTracingPipelineCreateInfo, RayTracing};
use crate::render::backend::shader::Shader;
use crate::render::backend::vulkan::descriptors::descriptor_set_layout::VkDescriptorSetLayout;
use crate::render::backend::vulkan::device::VkDevice;
use crate::render::backend::vulkan::shader::VkShader;
use std::marker::PhantomData;
use std::sync::Arc;

pub(crate) struct GraphicsCreateInfo {
    shaders: Vec<Arc<VkShader>>,
    bindings_descriptions: Vec<ash::vk::VertexInputBindingDescription>,
    attribute_descriptions: Vec<ash::vk::VertexInputAttributeDescription>,
    extent: ash::vk::Extent2D,
    topology: ash::vk::PrimitiveTopology,
    cull_mode: ash::vk::CullModeFlags,
    enable_depth_test: bool,
    depth_clamp: bool,
    blending_enable: bool,
    descriptor_set_layouts: Vec<Arc<VkDescriptorSetLayout>>,
    push_constants: Vec<ash::vk::PushConstantRange>,
    render_pass: ash::vk::RenderPass,
    color_attachments_count: u32,

    #[cfg(debug_assertions)]
    debug_name: std::ffi::CString,
}

pub(crate) struct ComputeCreateInfo {
    shader: Arc<VkShader>,
    descriptor_set_layouts: Vec<Arc<VkDescriptorSetLayout>>,
    push_constants: Vec<ash::vk::PushConstantRange>,

    #[cfg(debug_assertions)]
    debug_name: std::ffi::CString,
}

#[cfg(feature = "ray-tracing")]
pub(crate) struct RayTracingCreateInfo {
    ray_gen_shaders: Vec<Arc<VkShader>>,
    closest_hit_shaders: Vec<Arc<VkShader>>,
    miss_shaders: Vec<Arc<VkShader>>,
    descriptor_set_layouts: Vec<Arc<VkDescriptorSetLayout>>,
    push_constants: Vec<ash::vk::PushConstantRange>,

    #[cfg(debug_assertions)]
    debug_name: std::ffi::CString,
}

impl From<AttributeType> for ash::vk::Format {
    fn from(value: AttributeType) -> Self {
        match value {
            AttributeType::Float32 => ash::vk::Format::R32_SFLOAT,
            AttributeType::Float32x2 => ash::vk::Format::R32G32_SFLOAT,
            AttributeType::Float32x3 => ash::vk::Format::R32G32B32_SFLOAT,
            AttributeType::Float32x4 => ash::vk::Format::R32G32B32A32_SFLOAT,
        }
    }
}

impl From<Topology> for ash::vk::PrimitiveTopology {
    fn from(value: Topology) -> Self {
        match value {
            Topology::Point => ash::vk::PrimitiveTopology::POINT_LIST,
            Topology::Line => ash::vk::PrimitiveTopology::LINE_LIST,
            Topology::LineStrip => ash::vk::PrimitiveTopology::LINE_STRIP,
            Topology::Triangle => ash::vk::PrimitiveTopology::TRIANGLE_LIST,
            Topology::TriangleStrip => ash::vk::PrimitiveTopology::TRIANGLE_STRIP,
        }
    }
}

impl From<CullMode> for ash::vk::CullModeFlags {
    fn from(value: CullMode) -> Self {
        match value {
            CullMode::None => ash::vk::CullModeFlags::NONE,
            CullMode::Front => ash::vk::CullModeFlags::FRONT,
            CullMode::Back => ash::vk::CullModeFlags::BACK,
            CullMode::Both => ash::vk::CullModeFlags::FRONT_AND_BACK,
        }
    }
}

impl From<PushConstant> for ash::vk::PushConstantRange {
    fn from(value: PushConstant) -> Self {
        ash::vk::PushConstantRange {
            stage_flags: ash::vk::ShaderStageFlags::from_raw(value.shader.bits()),
            offset: value.offset,
            size: value.size,
        }
    }
}

fn attributes(
    attributes: Vec<AttributeType>,
) -> (
    Vec<ash::vk::VertexInputAttributeDescription>,
    Vec<ash::vk::VertexInputBindingDescription>,
) {
    let mut vk_attributes = Vec::with_capacity(attributes.len());

    let mut offset = 0;
    for (location, data) in attributes.into_iter().enumerate() {
        vk_attributes.push(ash::vk::VertexInputAttributeDescription {
            location: location as u32,
            binding: 0,
            format: data.into(),
            offset,
        });
        offset += match data {
            AttributeType::Float32 => 4,
            AttributeType::Float32x2 => 8,
            AttributeType::Float32x3 => 12,
            AttributeType::Float32x4 => 16,
        };
    }

    let binding_description = vec![ash::vk::VertexInputBindingDescription {
        binding: 0,
        stride: offset,
        input_rate: ash::vk::VertexInputRate::VERTEX,
    }];

    (vk_attributes, binding_description)
}

impl From<MVGraphicsPipelineCreateInfo> for GraphicsCreateInfo {
    fn from(value: MVGraphicsPipelineCreateInfo) -> Self {
        let (attribute_descriptions, bindings_descriptions) = attributes(value.attributes);
        GraphicsCreateInfo {
            shaders: value.shaders.into_iter().map(Shader::into_vulkan).collect(),
            bindings_descriptions,
            attribute_descriptions,
            extent: value.extent.into(),
            topology: value.topology.into(),
            cull_mode: value.cull_mode.into(),
            enable_depth_test: value.enable_depth_test,
            depth_clamp: value.depth_clamp,
            blending_enable: value.blending_enable,
            descriptor_set_layouts: value
                .descriptor_sets
                .into_iter()
                .map(|layout| layout.into_vulkan())
                .collect(),
            push_constants: value.push_constants.into_iter().map(Into::into).collect(),
            render_pass: value.framebuffer.as_vulkan().get_render_pass(),
            color_attachments_count: value.color_attachments_count,

            #[cfg(debug_assertions)]
            debug_name: crate::render::backend::to_ascii_cstring(value.label.unwrap_or_default()),
        }
    }
}

impl From<MVComputePipelineCreateInfo> for ComputeCreateInfo {
    fn from(value: MVComputePipelineCreateInfo) -> Self {
        ComputeCreateInfo {
            shader: value.shader.into_vulkan(),
            descriptor_set_layouts: value
                .descriptor_sets
                .into_iter()
                .map(|layout| layout.into_vulkan())
                .collect(),
            push_constants: value.push_constants.into_iter().map(Into::into).collect(),

            #[cfg(debug_assertions)]
            debug_name: crate::render::backend::to_ascii_cstring(value.label.unwrap_or_default()),
        }
    }
}

#[cfg(feature = "ray-tracing")]
impl From<MVRayTracingPipelineCreateInfo> for RayTracingCreateInfo {
    fn from(value: MVRayTracingPipelineCreateInfo) -> Self {
        RayTracingCreateInfo {
            ray_gen_shaders: value
                .ray_gen_shaders
                .into_iter()
                .map(Shader::into_vulkan)
                .collect(),
            closest_hit_shaders: value
                .closest_hit_shaders
                .into_iter()
                .map(Shader::into_vulkan)
                .collect(),
            miss_shaders: value
                .miss_shaders
                .into_iter()
                .map(Shader::into_vulkan)
                .collect(),
            descriptor_set_layouts: value
                .descriptor_sets
                .into_iter()
                .map(|layout| layout.into_vulkan())
                .collect(),
            push_constants: value.push_constants.into_iter().map(Into::into).collect(),

            #[cfg(debug_assertions)]
            debug_name: to_ascii_cstring(value.label.unwrap_or_default()),
        }
    }
}

struct PipelineConfigInfo<'a> {
    input_assembly_info: ash::vk::PipelineInputAssemblyStateCreateInfoBuilder<'a>,
    rasterization_info: ash::vk::PipelineRasterizationStateCreateInfoBuilder<'a>,
    multisample_info: ash::vk::PipelineMultisampleStateCreateInfoBuilder<'a>,
    color_blend_attachments: Vec<ash::vk::PipelineColorBlendAttachmentState>,
    depth_stencil_info: ash::vk::PipelineDepthStencilStateCreateInfoBuilder<'a>,
    vertex_input_info: ash::vk::PipelineVertexInputStateCreateInfoBuilder<'a>,
}

pub struct VkPipeline<Type: PipelineType = Graphics> {
    device: Arc<VkDevice>,
    handle: ash::vk::Pipeline,
    layout: ash::vk::PipelineLayout,
    _phantom: PhantomData<Type>,
}

impl<Type: PipelineType> VkPipeline<Type> {
    fn create_pipeline_layout(
        device: &Arc<VkDevice>,
        descriptor_set_layouts: &[Arc<VkDescriptorSetLayout>],
        push_constants: &[ash::vk::PushConstantRange],
    ) -> ash::vk::PipelineLayout {
        let layouts = descriptor_set_layouts
            .iter()
            .map(|layout| layout.get_layout())
            .collect::<Vec<_>>();
        let layout_info = ash::vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&layouts)
            .push_constant_ranges(push_constants);

        unsafe {
            device
                .get_device()
                .create_pipeline_layout(&layout_info, None)
        }
            .unwrap_or_else(|e| {
                log::error!("Failed to uix pipeline layout for error: {e}");
                panic!();
            })
    }

    pub(crate) fn get_handle(&self) -> ash::vk::Pipeline {
        self.handle
    }

    pub(crate) fn get_layout(&self) -> ash::vk::PipelineLayout {
        self.layout
    }
}

impl VkPipeline {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: GraphicsCreateInfo) -> Self {
        let layout = Self::create_pipeline_layout(
            &device,
            &create_info.descriptor_set_layouts,
            &create_info.push_constants,
        );
        let config_info = Self::create_pipeline_config_info(&create_info);

        let mut shader_stages = Vec::new();

        for shader in &create_info.shaders {
            shader_stages.push(shader.create_stage_create_info());
        }

        let viewports = [ash::vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: create_info.extent.width as f32,
            height: create_info.extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [ash::vk::Rect2D {
            offset: ash::vk::Offset2D { x: 0, y: 0 },
            extent: ash::vk::Extent2D {
                width: create_info.extent.width,
                height: create_info.extent.height,
            },
        }];

        let viewport_state_info = ash::vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .viewports(&viewports)
            .scissor_count(1)
            .scissors(&scissors);

        let dynamic_state_info =
            ash::vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&[
                ash::vk::DynamicState::SCISSOR,
                ash::vk::DynamicState::VIEWPORT,
            ]);

        let color_blend_info = ash::vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(ash::vk::LogicOp::COPY)
            .attachments(&config_info.color_blend_attachments);

        let graphics_create_info = ash::vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&config_info.vertex_input_info)
            .input_assembly_state(&config_info.input_assembly_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&config_info.rasterization_info)
            .multisample_state(&config_info.multisample_info)
            .color_blend_state(&color_blend_info)
            .dynamic_state(&dynamic_state_info)
            .depth_stencil_state(&config_info.depth_stencil_info)
            .layout(layout)
            .render_pass(create_info.render_pass)
            .subpass(0);

        let vk_info = [*graphics_create_info];

        let pipeline = unsafe {
            device.get_device().create_graphics_pipelines(
                ash::vk::PipelineCache::null(),
                &vk_info,
                None,
            )
        }
            .unwrap_or_else(|(_, e)| {
                log::error!("Failed to uix pipeline! error: {e}");
                panic!();
            })[0];

        #[cfg(debug_assertions)]
        device.set_object_name(
            &ash::vk::ObjectType::PIPELINE,
            ash::vk::Handle::as_raw(pipeline),
            create_info.debug_name.as_c_str(),
        );

        Self {
            device,
            handle: pipeline,
            layout,
            _phantom: Default::default(),
        }
    }

    fn create_pipeline_config_info(create_info: &GraphicsCreateInfo) -> PipelineConfigInfo {
        let enable_primitive_restart = create_info.topology
            == ash::vk::PrimitiveTopology::LINE_STRIP
            || create_info.topology == ash::vk::PrimitiveTopology::TRIANGLE_STRIP;
        let input_assembly_info = ash::vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(create_info.topology)
            .primitive_restart_enable(enable_primitive_restart);

        let rasterization_info = ash::vk::PipelineRasterizationStateCreateInfo::builder()
            .cull_mode(create_info.cull_mode)
            .depth_clamp_enable(create_info.depth_clamp)
            .rasterizer_discard_enable(false)
            .polygon_mode(ash::vk::PolygonMode::FILL)
            .line_width(1.0f32)
            .front_face(ash::vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisample_info = ash::vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(ash::vk::SampleCountFlags::TYPE_1) // rn just one sample
            .min_sample_shading(1.0f32)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);

        let mut color_blend_attachments = Vec::new();
        for i in 0..create_info.color_attachments_count {
            color_blend_attachments.push(ash::vk::PipelineColorBlendAttachmentState {
                blend_enable: create_info.blending_enable as u32,
                src_color_blend_factor: ash::vk::BlendFactor::SRC_ALPHA,
                dst_color_blend_factor: ash::vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
                color_blend_op: ash::vk::BlendOp::ADD,
                src_alpha_blend_factor: ash::vk::BlendFactor::SRC_ALPHA,
                dst_alpha_blend_factor: ash::vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
                alpha_blend_op: ash::vk::BlendOp::ADD,
                color_write_mask: ash::vk::ColorComponentFlags::RGBA,
            })
        }

        let depth_stencil_info = ash::vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(create_info.enable_depth_test)
            .depth_write_enable(create_info.enable_depth_test)
            .depth_compare_op(ash::vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.0f32)
            .max_depth_bounds(1.0f32)
            .stencil_test_enable(false); // we'll get back to that

        let vertex_input_info = ash::vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&create_info.attribute_descriptions)
            .vertex_binding_descriptions(&create_info.bindings_descriptions);

        PipelineConfigInfo {
            input_assembly_info,
            rasterization_info,
            multisample_info,
            color_blend_attachments,
            depth_stencil_info,
            vertex_input_info,
        }
    }

    pub(crate) fn bind(&self, command_buffer: ash::vk::CommandBuffer) {
        unsafe {
            self.device.get_device().cmd_bind_pipeline(
                command_buffer,
                ash::vk::PipelineBindPoint::GRAPHICS,
                self.handle,
            )
        };
    }
}

impl VkPipeline<Compute> {
    pub(crate) fn new(device: Arc<VkDevice>, create_info: ComputeCreateInfo) -> Self {
        let layout = Self::create_pipeline_layout(
            &device,
            &create_info.descriptor_set_layouts,
            &create_info.push_constants,
        );
        let compute_info = ash::vk::ComputePipelineCreateInfo::builder()
            .layout(layout)
            .stage(create_info.shader.create_stage_create_info());

        let vk_info = [*compute_info];

        let pipeline = unsafe {
            device.get_device().create_compute_pipelines(
                ash::vk::PipelineCache::null(),
                &vk_info,
                None,
            )
        }
            .unwrap_or_else(|e| {
                log::error!("Failed to uix pipeline! error: {}", e.1);
                panic!()
            })[0];

        #[cfg(debug_assertions)]
        device.set_object_name(
            &ash::vk::ObjectType::PIPELINE,
            ash::vk::Handle::as_raw(pipeline),
            create_info.debug_name.as_c_str(),
        );

        Self {
            device,
            handle: pipeline,
            layout,
            _phantom: Default::default(),
        }
    }

    pub(crate) fn bind(&self, command_buffer: ash::vk::CommandBuffer) {
        unsafe {
            self.device.get_device().cmd_bind_pipeline(
                command_buffer,
                ash::vk::PipelineBindPoint::COMPUTE,
                self.handle,
            )
        };
    }
}

#[cfg(feature = "ray-tracing")]
impl VkPipeline<RayTracing> {
    pub(crate) fn bind(&self, command_buffer: ash::vk::CommandBuffer) {
        unsafe {
            self.device.get_device().cmd_bind_pipeline(
                command_buffer,
                ash::vk::PipelineBindPoint::RAY_TRACING_KHR,
                self.handle,
            )
        };
    }
}

impl<Type: PipelineType> Drop for VkPipeline<Type> {
    fn drop(&mut self) {
        unsafe { self.device.get_device().destroy_pipeline(self.handle, None) };
        unsafe {
            self.device
                .get_device()
                .destroy_pipeline_layout(self.layout, None)
        };
    }
}
