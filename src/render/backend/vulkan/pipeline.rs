use std::ffi::CString;
use std::marker::PhantomData;
use std::sync::Arc;
use ash::vk::Handle;
use crate::err::panic;
use crate::render::backend::descriptor_set::DescriptorSetLayout;
use crate::render::backend::pipeline::{AttributeType, Compute, CullMode, Graphics, MVComputePipelineCreateInfo, MVGraphicsPipelineCreateInfo, PipelineType, PushConstant, Topology, VertexInputRate};
use crate::render::backend::shader::Shader;
use crate::render::backend::to_ascii_cstring;
use crate::render::backend::vulkan::device::VkDevice;
use crate::render::backend::vulkan::shader::{CreateInfo, VkShader};
#[cfg(feature = "ray-tracing")]
use crate::render::backend::pipeline::{RayTracing, MVRayTracingPipelineCreateInfo};

pub(crate) struct GraphicsCreateInfo {
    shaders: Vec<VkShader>,
    bindings_descriptions: Vec<ash::vk::VertexInputBindingDescription>,
    attribute_descriptions: Vec<ash::vk::VertexInputAttributeDescription>,
    extent: ash::vk::Extent2D,
    topology: ash::vk::PrimitiveTopology,
    cull_mode: ash::vk::CullModeFlags,
    enable_depth_test: bool,
    depth_clamp: bool,
    blending_enable: bool,
    descriptor_set_layouts: Vec<ash::vk::DescriptorSetLayout>,
    push_constants: Vec<ash::vk::PushConstantRange>,
    render_pass: ash::vk::RenderPass,
    color_attachments_count: u32,

    #[cfg(debug_assertions)]
    debug_name: CString
}

pub(crate) struct ComputeCreateInfo {
    shader: VkShader,
    descriptor_set_layouts: Vec<ash::vk::DescriptorSetLayout>,
    push_constants: Vec<ash::vk::PushConstantRange>,

    #[cfg(debug_assertions)]
    debug_name: CString
}

#[cfg(feature = "ray-tracing")]
pub(crate) struct RayTracingCreateInfo {
    ray_gen_shaders: Vec<VkShader>,
    closest_hit_shaders: Vec<VkShader>,
    miss_shaders: Vec<VkShader>,
    descriptor_set_layouts: Vec<ash::vk::DescriptorSetLayout>,
    push_constants: Vec<ash::vk::PushConstantRange>,

    #[cfg(debug_assertions)]
    debug_name: CString
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
            stage_flags: value.shader.into(),
            offset: value.offset,
            size: value.size,
        }
    }
}

impl From<MVGraphicsPipelineCreateInfo> for GraphicsCreateInfo {
    fn from(value: MVGraphicsPipelineCreateInfo) -> Self {
        GraphicsCreateInfo {
            shaders: value.shaders.into_iter().map(Shader::into_vulkan).collect(),
            bindings_descriptions: value.bindings.into_iter().map(|binding| ash::vk::VertexInputBindingDescription {
                binding: binding.binding,
                stride: binding.stride,
                input_rate: match binding.input_rate {
                    VertexInputRate::Vertex => ash::vk::VertexInputRate::VERTEX,
                    VertexInputRate::Instance => ash::vk::VertexInputRate::INSTANCE,
                },
            }).collect(),
            attribute_descriptions: value.attributes.into_iter().map(|attribute| ash::vk::VertexInputAttributeDescription {
                location: attribute.location,
                binding: attribute.binding,
                format: attribute.ty.into(),
                offset: attribute.offset,
            }).collect(),
            extent: value.extent.into(),
            topology: value.topology.into(),
            cull_mode: value.cull_mode.into(),
            enable_depth_test: value.enable_depth_test,
            depth_clamp: value.depth_clamp,
            blending_enable: value.blending_enable,
            descriptor_set_layouts: value.descriptor_sets.into_iter().map(DescriptorSetLayout::into_vulkan).collect(),
            push_constants: value.push_constants.into_iter().map(Into::into).collect(),
            render_pass: value.render_pass.as_vulkan(),
            color_attachments_count: value.color_attachments_count,

            #[cfg(debug_assertions)]
            debug_name: to_ascii_cstring(value.label.unwrap_or("".to_string())),
        }
    }
}

impl From<MVComputePipelineCreateInfo> for ComputeCreateInfo {
    fn from(value: MVComputePipelineCreateInfo) -> Self {
        ComputeCreateInfo {
            shader: value.shader.into_vulkan(),
            descriptor_set_layouts: value.descriptor_sets.into_iter().map(DescriptorSetLayout::into_vulkan).collect(),
            push_constants: value.push_constants.into_iter().map(Into::into).collect(),

            #[cfg(debug_assertions)]
            debug_name: to_ascii_cstring(value.label.unwrap_or("".to_string())),
        }
    }
}

#[cfg(feature = "ray-tracing")]
impl From<MVRayTracingPipelineCreateInfo> for RayTracingCreateInfo {
    fn from(value: MVRayTracingPipelineCreateInfo) -> Self {
        RayTracingCreateInfo {
            ray_gen_shaders: value.ray_gen_shaders.into_iter().map(Shader::into_vulkan).collect(),
            closest_hit_shaders: value.closest_hit_shaders.into_iter().map(Shader::into_vulkan).collect(),
            miss_shaders: value.miss_shaders.into_iter().map(Shader::into_vulkan).collect(),
            descriptor_set_layouts: value.descriptor_sets.into_iter().map(DescriptorSetLayout::into_vulkan).collect(),
            push_constants: value.push_constants.into_iter().map(Into::into).collect(),

            #[cfg(debug_assertions)]
            debug_name: to_ascii_cstring(value.label.unwrap_or("".to_string())),
        }
    }
}

struct PipelineConfigInfo {
    input_assembly_info: ash::vk::PipelineInputAssemblyStateCreateInfo,
    rasterization_info: ash::vk::PipelineRasterizationStateCreateInfo,
    multisample_info: ash::vk::PipelineMultisampleStateCreateInfo,
    color_blend_attachments: Vec<ash::vk::PipelineColorBlendAttachmentState>,
    color_blend_info: ash::vk::PipelineColorBlendStateCreateInfo,
    depth_stencil_info: ash::vk::PipelineDepthStencilStateCreateInfo,
    vertex_input_info: ash::vk::PipelineVertexInputStateCreateInfo,
    viewport_info: ash::vk::PipelineViewportStateCreateInfo,
    dynamic_state_info: ash::vk::PipelineDynamicStateCreateInfo
}

pub(crate) struct VkPipeline<Type: PipelineType = Graphics> {
    device: Arc<VkDevice>,
    handle: ash::vk::Pipeline,
    layout: ash::vk::PipelineLayout,
    _phantom: PhantomData<Type>,
}

impl<Type: PipelineType> VkPipeline<Type> {
    fn create_pipeline_layout(device: &Arc<VkDevice>, descriptor_set_layouts: &[ash::vk::DescriptorSetLayout], push_constants: &[ash::vk::PushConstantRange]) -> ash::vk::PipelineLayout {
        let layout_info = ash::vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&descriptor_set_layouts)
            .push_constant_ranges(&push_constants)
            .build();

        unsafe { device.get_device().create_pipeline_layout(&layout_info, None)}.unwrap_or_else(|e| {
            log::error!("Failed to create pipeline layout for error: {e}");
            panic!();
        })
    }
}

impl VkPipeline {
    pub fn new(device: Arc<VkDevice>, create_info: GraphicsCreateInfo) -> Self {
        let layout = Self::create_pipeline_layout(&device, &create_info.descriptor_set_layouts, &create_info.push_constants);
        let config_info = Self::create_pipeline_config_info(&create_info);

        let mut shader_stages = Vec::new();

        for shader in create_info.shaders {
            shader_stages.push(shader.create_stage_create_info());
        }

        let graphics_create_info = [ash::vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&config_info.vertex_input_info)
            .input_assembly_state(&config_info.input_assembly_info)
            .viewport_state(&config_info.viewport_info)
            .rasterization_state(&config_info.rasterization_info)
            .multisample_state(&config_info.multisample_info)
            .color_blend_state(&config_info.color_blend_info)
            .dynamic_state(&config_info.dynamic_state_info)
            .depth_stencil_state(&config_info.depth_stencil_info)
            .layout(layout)
            .render_pass(create_info.render_pass)
            .subpass(0)
            .build()];

        let pipeline = unsafe { device.get_device().create_graphics_pipelines(ash::vk::PipelineCache::null(), &graphics_create_info, None) }.unwrap_or_else(|(_, e)| {
            log::error!("Failed to create pipeline! error: {e}");
            panic!();
        })[0];

        device.set_object_name(&ash::vk::ObjectType::PIPELINE, pipeline.as_raw(), create_info.debug_name.as_c_str());

        Self {
            device,
            handle: pipeline,
            layout,
            _phantom: Default::default(),
        }
    }

    fn create_pipeline_config_info(create_info: &GraphicsCreateInfo) -> PipelineConfigInfo {

        let enable_primitive_restart = create_info.topology == ash::vk::PrimitiveTopology::LINE_STRIP || create_info.topology == ash::vk::PrimitiveTopology::TRIANGLE_STRIP;
        let input_assembly_info = ash::vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(create_info.topology)
            .primitive_restart_enable(enable_primitive_restart)
            .build();

        let viewport_info = [ash::vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(create_info.extent.width as f32)
            .height(create_info.extent.height as f32)
            .min_depth(0.0f32)
            .max_depth(1.0f32)
            .build()];

        let offset = ash::vk::Offset2D::builder().x(0).y(0).build();
        let extent = ash::vk::Extent2D::builder().height(create_info.extent.width).width(create_info.extent.height).build();
        let scissor_info = [ash::vk::Rect2D::builder()
            .offset(offset)
            .extent(extent)
            .build()];

        let rasterization_info = ash::vk::PipelineRasterizationStateCreateInfo::builder()
            .cull_mode(create_info.cull_mode)
            .depth_clamp_enable(create_info.depth_clamp)
            .rasterizer_discard_enable(false)
            .polygon_mode(ash::vk::PolygonMode::FILL)
            .line_width(1.0f32)
            .front_face(ash::vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .build();

        let multisample_info = ash::vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(ash::vk::SampleCountFlags::TYPE_1) // rn just one sample
            .min_sample_shading(1.0f32)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false)
            .build();

        let mut color_blend_attachments = Vec::new();
        for i in 0..create_info.color_attachments_count {
            color_blend_attachments.push(ash::vk::PipelineColorBlendAttachmentState::builder()
                .color_write_mask(ash::vk::ColorComponentFlags::RGBA)
                .blend_enable(create_info.blending_enable)
                .src_alpha_blend_factor(ash::vk::BlendFactor::SRC_ALPHA)
                .dst_color_blend_factor(ash::vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .color_blend_op(ash::vk::BlendOp::ADD)
                .src_alpha_blend_factor(ash::vk::BlendFactor::ONE)
                .dst_alpha_blend_factor(ash::vk::BlendFactor::ZERO)
                .alpha_blend_op(ash::vk::BlendOp::ADD)
                .build()
            )
        }

        let color_blend_info = ash::vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(ash::vk::LogicOp::COPY)
            .attachments(&color_blend_attachments)
            .build();

        let depth_stencil_info = ash::vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(create_info.enable_depth_test)
            .depth_write_enable(create_info.enable_depth_test)
            .depth_compare_op(ash::vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.0f32)
            .max_depth_bounds(1.0f32)
            .stencil_test_enable(false) // we'll get back to that
            .build();

        let vertex_input_info = ash::vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&create_info.attribute_descriptions)
            .vertex_binding_descriptions(&create_info.bindings_descriptions)
            .build();

        let viewport_state_info = ash::vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .viewports(&viewport_info)
            .scissor_count(1)
            .scissors(&scissor_info)
            .build();

        let dynamic_states = [ash::vk::DynamicState::SCISSOR, ash::vk::DynamicState::VIEWPORT];

        let dynamic_state_info = ash::vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states)
            .build();

        PipelineConfigInfo {
            input_assembly_info,
            rasterization_info,
            multisample_info,
            color_blend_attachments,
            color_blend_info,
            depth_stencil_info,
            vertex_input_info,
            viewport_info: viewport_state_info,
            dynamic_state_info
        }
    }
}

impl VkPipeline<Compute> {
    pub fn new(device: Arc<VkDevice>, create_info: ComputeCreateInfo) -> Self {
        let layout = Self::create_pipeline_layout(&device, &create_info.descriptor_set_layouts, &create_info.push_constants);
        let compute_info = [ash::vk::ComputePipelineCreateInfo::builder()
            .layout(layout)
            .stage(create_info.shader.create_stage_create_info())
            .build()];

        let pipeline = unsafe { device.get_device().create_compute_pipelines(ash::vk::PipelineCache::null(), &compute_info, None)}.unwrap_or_else(|e| {
            log::error!("Failed to create pipeline! error: {}", e.1);
            panic!()
        })[0];

        Self {
            device,
            handle: pipeline,
            layout,
            _phantom: Default::default(),
        }
    }
}

#[cfg(feature = "ray-tracing")]
impl VkPipeline<RayTracing> {

}

