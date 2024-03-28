use bitflags::Flags;
use glam::Mat4;
use shaderc::{OptimizationLevel, ShaderKind, TargetEnv};
use crate::render::backend::buffer::{Buffer, BufferUsage, MemoryProperties, MVBufferCreateInfo};
use crate::render::backend::command_buffer::CommandBuffer;
use crate::render::backend::descriptor_set::{DescriptorPool, DescriptorPoolFlags, DescriptorPoolSize, DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorType, MVDescriptorPoolCreateInfo, MVDescriptorSetFromLayoutCreateInfo, MVDescriptorSetLayoutCreateInfo};
use crate::render::backend::device::Device;
use crate::render::backend::Extent2D;
use crate::render::backend::framebuffer::ClearColor;
use crate::render::backend::pipeline::{AttributeType, Compute, CullMode, Graphics, MVGraphicsPipelineCreateInfo, Pipeline, Topology};
use crate::render::backend::shader::{MVShaderCreateInfo, Shader, ShaderStage};
use crate::render::state::State;
use crate::render::window::WindowCreateInfo;

struct GraphicsPipeline2d {
    vertex: Shader,
    geometry: Option<Shader>,
    fragment: Shader,

    attributes: Vec<AttributeType>,

    matrix_layout: DescriptorSetLayout,

    pipeline: Pipeline<Graphics>,
    label: String,
}

impl GraphicsPipeline2d {
    fn new(state: &State, info: &WindowCreateInfo, matrix_layout: DescriptorSetLayout, vertex: Shader, geometry: Option<Shader>, fragment: Shader, attributes: Vec<AttributeType>, #[cfg(debug_assertions)] label: &'static str) -> Self {
        let label = label.to_string();

        let mut shaders = vec![vertex.clone(), fragment.clone()];
        if let Some(geometry) = geometry.clone() {
            shaders.push(geometry);
        }

        let pipeline = Pipeline::<Graphics>::new(state.get_device(), MVGraphicsPipelineCreateInfo {
            shaders,
            attributes: attributes.clone(),
            extent: Extent2D {
                width: info.width,
                height: info.height,
            },
            topology: geometry.is_some().then_some(Topology::Point).unwrap_or(Topology::Triangle),
            cull_mode: CullMode::Back,
            enable_depth_test: false,
            depth_clamp: false,
            blending_enable: true,
            descriptor_sets: vec![matrix_layout.clone()],
            push_constants: vec![],
            framebuffer: state.get_current_framebuffer(),
            color_attachments_count: 1,
            label: Some(label.clone()),
        });

        GraphicsPipeline2d {
            vertex,
            geometry,
            fragment,
            attributes,
            matrix_layout,
            pipeline,
            label
        }
    }

    fn resize(&mut self, state: &State, width: u32, height: u32) {
        let mut shaders = vec![self.vertex.clone(), self.fragment.clone()];
        if let Some(geometry) = self.geometry.clone() {
            shaders.push(geometry);
        }

        self.pipeline = Pipeline::<Graphics>::new(state.get_device(), MVGraphicsPipelineCreateInfo {
            shaders,
            attributes: self.attributes.clone(),
            extent: Extent2D {
                width,
                height,
            },
            topology: self.geometry.is_some().then_some(Topology::Point).unwrap_or(Topology::Triangle),
            cull_mode: CullMode::Back,
            enable_depth_test: false,
            depth_clamp: false,
            blending_enable: true,
            descriptor_sets: vec![self.matrix_layout.clone()],
            push_constants: vec![],
            framebuffer: state.get_current_framebuffer(),
            color_attachments_count: 1,
            label: Some(self.label.clone()),
        });
    }
}

struct EffectPipeline2d {
    shader: Shader,

    pipeline: Pipeline<Compute>
}

pub(crate) struct Render2d {
    descriptor_pool: DescriptorPool,
    matrix_set_layout: DescriptorSetLayout,
    matrix_sets: Vec<DescriptorSet>,
    matrix_buffers: Vec<Buffer>,
    vertex_buffer: Buffer,

    square_pipeline: GraphicsPipeline2d,
    default_pipeline: GraphicsPipeline2d,
}

impl Render2d {
    pub(crate) fn new(state: &State, info: &WindowCreateInfo) -> Self {
        //TODO: replace all shaders with premade SPIR-V when done with renderer
        fn compile(str: &str, name: &str, kind: ShaderKind) -> Vec<u32> {
            let compiler = shaderc::Compiler::new().expect("Failed to initialize shader compiler");
            let mut options =
                shaderc::CompileOptions::new().expect("Failed to initialize shader compiler");
            options.set_target_env(TargetEnv::Vulkan, ash::vk::API_VERSION_1_2);
            options.set_optimization_level(OptimizationLevel::Performance);
            let binary_result = compiler
                .compile_into_spirv(str, kind, name, "main", Some(&options)).unwrap();
            binary_result.as_binary().to_vec()
        }

        let square_vertex_shader = include_str!("shaders/2d/square.vert");
        let square_vert_compiled = compile(square_vertex_shader, "shaders/2d/square.vert", ShaderKind::Vertex);

        let geometry_shader = include_str!("shaders/2d/square.geom");
        let geom_compiled = compile(geometry_shader, "shaders/2d/square.geom", ShaderKind::Geometry);

        let vertex_shader = include_str!("shaders/2d/default.vert");
        let vert_compiled = compile(vertex_shader, "shaders/2d/default.vert", ShaderKind::Vertex);

        let fragment_shader = include_str!("shaders/2d/default.frag");
        let frag_compiled = compile(fragment_shader, "shaders/2d/default.frag", ShaderKind::Fragment);

        let square_vertex = Shader::new(state.get_device(), MVShaderCreateInfo {
            stage: ShaderStage::Vertex,
            code: square_vert_compiled,
            label: Some("Square 2d vertex shader".to_string()),
        });

        let geometry = Shader::new(state.get_device(), MVShaderCreateInfo {
            stage: ShaderStage::Geometry,
            code: geom_compiled,
            label: Some("Square geometry shader".to_string()),
        });

        let vertex = Shader::new(state.get_device(), MVShaderCreateInfo {
            stage: ShaderStage::Vertex,
            code: vert_compiled,
            label: Some("Default 2d vertex shader".to_string()),
        });

        let fragment = Shader::new(state.get_device(), MVShaderCreateInfo {
            stage: ShaderStage::Fragment,
            code: frag_compiled,
            label: Some("Default 2d fragment shader".to_string()),
        });

        let descriptor_pool = DescriptorPool::new(state.get_device(), MVDescriptorPoolCreateInfo {
            sizes: vec![
                DescriptorPoolSize {
                    ty: DescriptorType::UniformBuffer,
                    count: 1000,
                },
                DescriptorPoolSize {
                    ty: DescriptorType::CombinedImageSampler,
                    count: 1000,
                },
                DescriptorPoolSize {
                    ty: DescriptorType::StorageImage,
                    count: 1000,
                },
                DescriptorPoolSize {
                    ty: DescriptorType::StorageBuffer,
                    count: 1000,
                }
            ],
            max_sets: 1000,
            flags: DescriptorPoolFlags::FREE_DESCRIPTOR,
            label: Some("Descriptor pool 2d".to_string()),
        });

        let matrix_set_layout = DescriptorSetLayout::new(state.get_device(), MVDescriptorSetLayoutCreateInfo {
            bindings: vec![
                DescriptorSetLayoutBinding {
                    index: 0,
                    stages: ShaderStage::Vertex | ShaderStage::Geometry,
                    ty: DescriptorType::UniformBuffer,
                    count: 1,
                }
            ],
            label: Some("Matrix descriptor set layout 2d".to_string()),
        });

        let square_pipeline = GraphicsPipeline2d::new(
            state,
            info,
            matrix_set_layout.clone(),
            square_vertex,
            Some(geometry),
            fragment.clone(),
            vec![AttributeType::Float32x2, AttributeType::Float32x2, AttributeType::Float32x4, AttributeType::Float32, AttributeType::Float32x4],
            "Square pipeline 2d"
        );

        let default_pipeline = GraphicsPipeline2d::new(
            state,
            info,
            matrix_set_layout.clone(),
            vertex,
            None,
            fragment,
            vec![],
            "Default pipeline 2d"
        );

        let mut vertex_buffer = Buffer::new(state.get_device().clone(),
        MVBufferCreateInfo {
            instance_size: (2 + 2 + 4 + 1 + 4) * 2 * 4,
            instance_count: 1,
            buffer_usage: BufferUsage::VERTEX_BUFFER,
            memory_properties: MemoryProperties::DEVICE_LOCAL,
            minimum_alignment: 1,
            memory_usage: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
            label: Some("Vertex buffer".to_string()),
        });

        let mut matrix_buffers = Vec::new();
        let mut matrix_sets = Vec::new();

        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            matrix_buffers.push(Buffer::new(state.get_device(), MVBufferCreateInfo {
                instance_size: 128,
                instance_count: 1,
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_properties: MemoryProperties::HOST_VISIBLE | MemoryProperties::HOST_COHERENT,
                minimum_alignment: 1,
                memory_usage: gpu_alloc::UsageFlags::HOST_ACCESS,
                label: Some("Matrix buffer".to_string()),
            }));
        }

        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            matrix_sets.push(DescriptorSet::from_layout(state.get_device(), MVDescriptorSetFromLayoutCreateInfo {
                pool: descriptor_pool.clone(),
                layout: matrix_set_layout.clone(),
                label: Some("Matrix descriptor set 2d".to_string()),
            }));
            matrix_sets[index as usize].add_buffer(0,&matrix_buffers[index as usize], 0, matrix_buffers[index as usize].get_size());
            matrix_sets[index as usize].build();
        }

        let vertex_data =
            [
                100.0f32, -100.0, 150.0, 150.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,    // Left Square
                500.5, -100.0, 150.0, 150.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0      // Right Square
            ];

        let byte_data_vertex = unsafe {
            std::slice::from_raw_parts(vertex_data.as_ptr() as *const u8, vertex_data.len() * 4)
        };

        vertex_buffer.write(byte_data_vertex, 0, None);

        Render2d {
            descriptor_pool,
            matrix_set_layout,
            matrix_sets,
            square_pipeline,
            default_pipeline,
            vertex_buffer,
            matrix_buffers,
        }
    }

    pub(crate) fn resize(&mut self, state: &State, width: u32, height: u32) {
        self.square_pipeline.resize(state, width, height);
        self.default_pipeline.resize(state, width, height);
    }

    pub(crate) fn update_matrices(&mut self, state: &State, cmd: &CommandBuffer, view: Mat4, proj: Mat4) {
        let buffer = &mut self.matrix_buffers[state.get_current_frame_index() as usize];
        let matrices = [view, proj];
        let bytes = unsafe { std::slice::from_raw_parts(matrices.as_ptr() as *const u8, 128) };
        buffer.write(bytes, 0, Some(cmd));
    }

    pub(crate) fn draw(&mut self, state: &State, cmd: &CommandBuffer) {
        let swapchain = state.get_swapchain();
        let framebuffer = swapchain.get_current_framebuffer();
        let matrix_set = &mut self.matrix_sets[swapchain.get_current_frame() as usize];

        framebuffer.begin_render_pass(cmd, &[ClearColor::Color([0.1, 0.1, 0.1, 1.0])], swapchain.get_extent());

        self.square_pipeline.pipeline.bind(cmd);

        cmd.bind_vertex_buffer(&self.vertex_buffer);

        matrix_set.bind(&cmd, &self.square_pipeline.pipeline, 0);

        cmd.draw(2, 0);

        framebuffer.end_render_pass(cmd);
    }
}