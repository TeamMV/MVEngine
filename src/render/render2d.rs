use std::sync::Arc;
use bitflags::Flags;
use glam::Mat4;
use gpu_alloc::UsageFlags;
use shaderc::{OptimizationLevel, ShaderKind, TargetEnv};
use crate::render::backend::buffer::{Buffer, BufferUsage, MemoryProperties, MVBufferCreateInfo};
use crate::render::backend::command_buffer::CommandBuffer;
use crate::render::backend::descriptor_set::{DescriptorPool, DescriptorPoolFlags, DescriptorPoolSize, DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorType, MVDescriptorPoolCreateInfo, MVDescriptorSetCreateInfo, MVDescriptorSetFromLayoutCreateInfo, MVDescriptorSetLayoutCreateInfo};
use crate::render::backend::device::Device;
use crate::render::backend::{Extent2D, Extent3D};
use crate::render::backend::framebuffer::{ClearColor, Framebuffer, MVFramebufferCreateInfo};
use crate::render::backend::image::{Image, ImageAspect, ImageFormat, ImageLayout, ImageTiling, ImageType, ImageUsage, MVImageCreateInfo};
use crate::render::backend::pipeline::{AttributeType, Compute, CullMode, Graphics, MVComputePipelineCreateInfo, MVGraphicsPipelineCreateInfo, Pipeline, Topology};
use crate::render::backend::sampler::{Filter, MipmapMode, MVSamplerCreateInfo, Sampler, SamplerAddressMode};
use crate::render::backend::shader::{MVShaderCreateInfo, Shader, ShaderStage};
use crate::render::state::State;
use crate::render::window::WindowCreateInfo;

struct GraphicsPipeline2d {
    vertex: Shader,
    geometry: Option<Shader>,
    fragment: Shader,
    framebuffer: Framebuffer,

    attributes: Vec<AttributeType>,

    matrix_layout: DescriptorSetLayout,

    pipeline: Pipeline<Graphics>,
    label: String,
}

impl GraphicsPipeline2d {
    fn new(state: &State, info: &WindowCreateInfo, matrix_layout: DescriptorSetLayout, vertex: Shader, geometry: Option<Shader>, fragment: Shader, attributes: Vec<AttributeType>, framebuffer: Framebuffer, #[cfg(debug_assertions)] label: &'static str) -> Self {
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
            framebuffer: framebuffer.clone(),
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
            label,
            framebuffer: framebuffer.clone()
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
            framebuffer: self.framebuffer.clone(),
            color_attachments_count: 1,
            label: Some(self.label.clone()),
        });
    }
}

struct EffectPipeline2d {
    shader: Shader,

    effect_set_layout: DescriptorSetLayout,
    image_set: DescriptorSet,
    input_image: Image,
    output_image: Image,
    textures: Arc<Vec<Image>>,

    pipeline: Pipeline<Compute>
}

impl EffectPipeline2d {
    fn new(state: &State, info: &WindowCreateInfo, shader: Shader, pool: &DescriptorPool, effect_set_layout: DescriptorSetLayout, sampler: &Sampler, input_image: Image, output_image: Image, input_textures: Arc<Vec<Image>>) -> Self {
        let mut bindings = Vec::new();
        bindings.push(DescriptorSetLayoutBinding{
            index: 0,
            stages: ShaderStage::Compute,
            ty: DescriptorType::CombinedImageSampler,
            count: 1,
        });
        bindings.push(DescriptorSetLayoutBinding{
            index: 1,
            stages: ShaderStage::Compute,
            ty: DescriptorType::StorageImage,
            count: 1,
        });

        for index in 0..input_textures.len() {
            bindings.push(DescriptorSetLayoutBinding{
                index: (index + 2) as u32,
                stages: ShaderStage::Compute,
                ty: DescriptorType::CombinedImageSampler,
                count: 1,
            });
        }

        let image_set_layout = DescriptorSetLayout::new(state.get_device().clone(), MVDescriptorSetLayoutCreateInfo {
            bindings,
            label: Some("Effect Pipeline image set".to_string()),
        });
        let mut image_set = DescriptorSet::from_layout(state.get_device().clone(), MVDescriptorSetFromLayoutCreateInfo{
            pool: pool.clone(),
            layout: image_set_layout.clone(),
            label: Some("Image Set".to_string()),
        });

        image_set.add_image(0, &input_image, sampler, ImageLayout::ShaderReadOnlyOptimal);
        image_set.add_image(1, &output_image, sampler, ImageLayout::General);

        for index in 0..input_textures.len() {
            image_set.add_image((index + 2) as u32, &input_textures[index], sampler, ImageLayout::ShaderReadOnlyOptimal);
        }

        image_set.build();

        let pipeline = Pipeline::<Compute>::new(state.get_device(), MVComputePipelineCreateInfo {
            shader: shader.clone(),
            descriptor_sets: vec![image_set_layout.clone(), effect_set_layout.clone()],
            push_constants: vec![],
            label: Some("Effect pipeline 2d".to_string()),
        });

        Self {
            shader,
            image_set,
            effect_set_layout,
            pipeline,
            input_image,
            output_image,
            textures: input_textures
        }
    }

    pub(crate) fn run(&mut self, cmd: &CommandBuffer, data_set: &mut DescriptorSet) {
        self.input_image.transition_layout(ImageLayout::ShaderReadOnlyOptimal, Some(cmd), ash::vk::AccessFlags::empty(), ash::vk::AccessFlags::empty());
        self.output_image.transition_layout(ImageLayout::General, Some(cmd), ash::vk::AccessFlags::empty(), ash::vk::AccessFlags::empty());

        self.pipeline.bind(cmd);

        self.image_set.bind(cmd, &self.pipeline, 0);
        data_set.bind(cmd, &self.pipeline, 1);

        cmd.dispatch(Extent3D{width: self.input_image.get_extent().width, height: self.input_image.get_extent().height, depth: 1});
    }

    pub(crate) fn update_images(&mut self, sampler: &Sampler, input_image: &Image, output_image: &Image, input_textures: Arc<Vec<Image>>) {
        self.input_image = input_image.clone();
        self.output_image = output_image.clone();

        self.image_set.update_image(0, input_image, sampler, ImageLayout::ShaderReadOnlyOptimal);
        self.image_set.update_image(1, output_image, sampler, ImageLayout::General);

        for index in 0..input_textures.len() {
            self.image_set.update_image((index + 2) as u32, &input_textures[index], sampler, ImageLayout::ShaderReadOnlyOptimal);
        }
    }
}

struct ToneMap {
    shader: Shader,
    image_set: DescriptorSet,
    input_image: Image,
    output_image: Image,
    pipeline: Pipeline<Compute>,
}

impl ToneMap {
    fn new(state: &State, info: &WindowCreateInfo, sampler: &Sampler, pool: DescriptorPool, shader: Shader, input_image: Image, output_image: Image) -> Self {
        let image_set_layout = DescriptorSetLayout::new(state.get_device().clone(), MVDescriptorSetLayoutCreateInfo {
            bindings: vec![DescriptorSetLayoutBinding{
                index: 0,
                stages: ShaderStage::Compute,
                ty: DescriptorType::CombinedImageSampler,
                count: 1,
            }, DescriptorSetLayoutBinding{
                index: 1,
                stages: ShaderStage::Compute,
                ty: DescriptorType::StorageImage,
                count: 1,
            }],
            label: Some("Effect Pipeline image set".to_string()),
        });
        
        let mut image_set= DescriptorSet::from_layout(state.get_device(), MVDescriptorSetFromLayoutCreateInfo{
            pool,
            layout: image_set_layout.clone(),
            label: Some("Tonemap Set".to_string()),
        });

        image_set.add_image(0, &input_image, sampler, ImageLayout::ShaderReadOnlyOptimal);
        image_set.add_image(1, &output_image, sampler, ImageLayout::General);

        image_set.build();
        
        let pipeline = Pipeline::<Compute>::new(state.get_device(), MVComputePipelineCreateInfo {
            shader: shader.clone(),
            descriptor_sets: vec![image_set_layout],
            push_constants: vec![],
            label: Some("Tone mapping pipeline 2d".to_string()),
        });

        Self {
            shader,
            image_set,
            pipeline,
            input_image,
            output_image,
        }
    }

    fn update_images(&mut self, sampler: &Sampler, input_image: &Image, output_image: &Image) {
        self.image_set.update_image(0, input_image, sampler, ImageLayout::ShaderReadOnlyOptimal);
        self.image_set.update_image(1, output_image, sampler, ImageLayout::General);

        self.input_image = input_image.clone();
        self.output_image = output_image.clone();
    }

    fn run(&mut self, cmd: &CommandBuffer) {
        self.input_image.transition_layout(ImageLayout::ShaderReadOnlyOptimal, Some(cmd), ash::vk::AccessFlags::empty(), ash::vk::AccessFlags::empty());
        self.output_image.transition_layout(ImageLayout::General, Some(cmd), ash::vk::AccessFlags::empty(), ash::vk::AccessFlags::empty());

        self.pipeline.bind(cmd);

        self.image_set.bind(cmd, &self.pipeline, 0);

        cmd.dispatch(Extent3D{width: self.input_image.get_extent().width, height: self.input_image.get_extent().height, depth: 1});
    }
}

struct Uniform {
    layout: DescriptorSetLayout,
    sets: Vec<DescriptorSet>,
    buffers: Vec<Buffer>,
}

pub(crate) struct Render2d {
    descriptor_pool: DescriptorPool,
    matrix_uniform: Uniform,
    vertex_buffer: Buffer,
    sampler: Sampler,

    square_pipeline: GraphicsPipeline2d,
    default_pipeline: GraphicsPipeline2d,

    tone_maps: Vec<ToneMap>,

    effect_images: Vec<Vec<Image>>,
    effects: Vec<EffectPipeline2d>,
    effect_uniform: Uniform,
    geometry_framebuffers: Vec<Framebuffer>,
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

        let invert_shader = include_str!("shaders/2d/invert.comp");
        let invert_compiled = compile(invert_shader, "shaders/2d/invert.comp", ShaderKind::Compute);

        let tonemap_shader = include_str!("shaders/2d/tonemap.comp");
        let tonemap_compiled = compile(tonemap_shader, "shaders/2d/tonemap.comp", ShaderKind::Compute);

        let square_vertex = Shader::new(state.get_device(), MVShaderCreateInfo {
            stage: ShaderStage::Vertex,
            code: square_vert_compiled,
            label: Some("Square vertex shader 2d".to_string()),
        });

        let geometry = Shader::new(state.get_device(), MVShaderCreateInfo {
            stage: ShaderStage::Geometry,
            code: geom_compiled,
            label: Some("Square geometry shader 2d".to_string()),
        });

        let vertex = Shader::new(state.get_device(), MVShaderCreateInfo {
            stage: ShaderStage::Vertex,
            code: vert_compiled,
            label: Some("Default vertex shader 2d".to_string()),
        });

        let fragment = Shader::new(state.get_device(), MVShaderCreateInfo {
            stage: ShaderStage::Fragment,
            code: frag_compiled,
            label: Some("Default fragment shader 2d".to_string()),
        });

        let invert = Shader::new(state.get_device(), MVShaderCreateInfo {
            stage: ShaderStage::Compute,
            code: invert_compiled,
            label: Some("Invert effect shader 2d".to_string()),
        });

        let tonemap = Shader::new(state.get_device(), MVShaderCreateInfo {
            stage: ShaderStage::Compute,
            code: tonemap_compiled,
            label: Some("Tonemap effect shader 2d".to_string()),
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

        let sampler = Sampler::new(state.get_device(), MVSamplerCreateInfo {
            address_mode: SamplerAddressMode::ClampToEdge,
            filter_mode: Filter::Linear,
            mipmap_mode: MipmapMode::Linear,
            label: Some("Sampler 2d".to_string()),
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
        let mut geometry_framebuffers = Vec::new();
        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            geometry_framebuffers.push(
                Framebuffer::new(state.get_device().clone(), MVFramebufferCreateInfo {
                    attachment_formats: vec![ImageFormat::R32G32B32A32],
                    extent: state.get_swapchain().get_extent(),
                    image_usage_flags: ImageUsage::SAMPLED,
                    render_pass_info: None,
                    label: Some("Geometry HDR framebuffer".to_string()),
                }));
        }

        let square_pipeline = GraphicsPipeline2d::new(
            state,
            info,
            matrix_set_layout.clone(),
            square_vertex,
            Some(geometry),
            fragment.clone(),
            vec![AttributeType::Float32x2, AttributeType::Float32x2, AttributeType::Float32x4, AttributeType::Float32, AttributeType::Float32x4],
            geometry_framebuffers[0].clone(),
            "Square pipeline 2d"
        );

        let default_pipeline = GraphicsPipeline2d::new(
            state,
            info,
            matrix_set_layout.clone(),
            vertex,
            None,
            fragment,
            vec![AttributeType::Float32x2, AttributeType::Float32x2, AttributeType::Float32x4, AttributeType::Float32],
            geometry_framebuffers[0].clone(),
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
            label: Some("Vertex buffer 2d".to_string()),
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
                label: Some("Matrix buffer 2d".to_string()),
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
                100.0f32, -100.0, 150.0, 150.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0,    // Left Square
                500.5, -100.0, 150.0, 150.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0      // Right Square
            ];

        let byte_data_vertex = unsafe {
            std::slice::from_raw_parts(vertex_data.as_ptr() as *const u8, vertex_data.len() * 4)
        };

        vertex_buffer.write(byte_data_vertex, 0, None);

        let effect_set_layout = DescriptorSetLayout::new(state.get_device(), MVDescriptorSetLayoutCreateInfo {
            bindings: vec![DescriptorSetLayoutBinding {
                index: 0,
                stages: ShaderStage::Compute,
                ty: DescriptorType::UniformBuffer,
                count: 1,
            }],
            label: Some("Effect set layout 2d".to_string()),
        });

        let mut effect_sets = Vec::new();
        let mut effect_buffers = Vec::new();

        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            effect_buffers.push(Buffer::new(state.get_device(), MVBufferCreateInfo {
                instance_size: 4,
                instance_count: 1,
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_properties: MemoryProperties::HOST_VISIBLE | MemoryProperties::HOST_COHERENT,
                minimum_alignment: 1,
                memory_usage: UsageFlags::HOST_ACCESS,
                label: Some("Effect buffer 2d".to_string()),
            }));
        }

        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            effect_sets.push(DescriptorSet::from_layout(state.get_device(), MVDescriptorSetFromLayoutCreateInfo {
                pool: descriptor_pool.clone(),
                layout: effect_set_layout.clone(),
                label: Some("Effect descriptor set 2d".to_string()),
            }));
            effect_sets[index as usize].add_buffer(0,&effect_buffers[index as usize], 0, effect_buffers[index as usize].get_size());
            effect_sets[index as usize].build();
        }

        let mut effects_images: Vec<Vec<Image>> = Vec::new();

        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            let mut image_pair = Vec::new();

            for index in 0..2 {
                image_pair.push(Image::new(state.get_device().clone(), MVImageCreateInfo{
                    size: state.get_swapchain().get_extent(),
                    format: ImageFormat::R8G8B8A8,
                    usage: ImageUsage::SAMPLED | ImageUsage::STORAGE,
                    memory_properties: MemoryProperties::DEVICE_LOCAL,
                    aspect: ImageAspect::COLOR,
                    tiling: ImageTiling::Optimal,
                    layer_count: 1,
                    image_type: ImageType::Image2D,
                    cubemap: false,
                    memory_usage_flags: UsageFlags::FAST_DEVICE_ACCESS,
                    data: None,
                    label: Some("Effect image 2d".to_string()),
                }));
            }
            effects_images.push(image_pair);
        }

        let mut effects = Vec::new();

        let mut tone_maps = Vec::new();
        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            effects.push(EffectPipeline2d::new(state, info, invert.clone(), &descriptor_pool, effect_set_layout.clone(), &sampler, effects_images[index as usize][0].clone(), state.get_swapchain().get_framebuffer(index as usize).get_image(0).clone(), Arc::new(Vec::new())));
            tone_maps.push(ToneMap::new(state, info, &sampler, descriptor_pool.clone(), tonemap.clone(), geometry_framebuffers[index as usize].get_image(0).clone(), effects_images[index as usize][0].clone()));
        }

        Render2d {
            descriptor_pool,
            square_pipeline,
            default_pipeline,
            vertex_buffer,
            matrix_uniform: Uniform {
                layout: matrix_set_layout,
                sets: matrix_sets,
                buffers: matrix_buffers,
            },
            effects,
            effect_uniform: Uniform {
                layout: effect_set_layout,
                sets: effect_sets,
                buffers: effect_buffers,
            },
            sampler,
            tone_maps,
            geometry_framebuffers,
            effect_images: effects_images
        }
    }

    pub(crate) fn resize(&mut self, state: &State, width: u32, height: u32) {
        state.get_device().wait_idle();

        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            self.geometry_framebuffers[index as usize] =
                Framebuffer::new(state.get_device().clone(), MVFramebufferCreateInfo {
                    attachment_formats: vec![ImageFormat::R32G32B32A32],
                    extent: state.get_swapchain().get_extent(),
                    image_usage_flags: ImageUsage::SAMPLED,
                    render_pass_info: None,
                    label: Some("Geometry HDR framebuffer".to_string()),
                });
        }

        self.square_pipeline.resize(state, width, height);
        self.default_pipeline.resize(state, width, height);

        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            self.effects[index as usize].update_images(&self.sampler, &self.effect_images[index as usize][0], &state.get_swapchain().get_framebuffer(index as usize).get_image(0), Arc::new(Vec::new()));
            self.tone_maps[index as usize].update_images(&self.sampler, &self.geometry_framebuffers[index as usize].get_image(0).clone(), &self.effect_images[index as usize][0].clone());
        }
    }

    pub(crate) fn update_matrices(&mut self, state: &State, cmd: &CommandBuffer, view: Mat4, proj: Mat4) {
        let buffer = &mut self.matrix_uniform.buffers[state.get_current_frame_index() as usize];
        let matrices = [view, proj];
        let bytes = unsafe { std::slice::from_raw_parts(matrices.as_ptr() as *const u8, 128) };
        buffer.write(bytes, 0, Some(cmd));
    }

    pub(crate) fn draw(&mut self, state: &State, cmd: &CommandBuffer) {
        let swapchain = state.get_swapchain();
        let swapchain_framebuffer = swapchain.get_current_framebuffer();
        let matrix_set = &mut self.matrix_uniform.sets[swapchain.get_current_frame() as usize];
        let current_frame = swapchain.get_current_frame();
        let geometry_framebuffer = &self.geometry_framebuffers[current_frame as usize];

        geometry_framebuffer.begin_render_pass(cmd, &[ClearColor::Color([0.0, 0.0, 0.0, 1.0])], swapchain.get_extent());

        self.square_pipeline.pipeline.bind(cmd);

        cmd.bind_vertex_buffer(&self.vertex_buffer);

        matrix_set.bind(&cmd, &self.square_pipeline.pipeline, 0);

        cmd.draw(2, 0);

        geometry_framebuffer.end_render_pass(cmd);

        self.tone_maps[current_frame as usize].run(cmd);

        self.effects[current_frame as usize].run(cmd, &mut self.effect_uniform.sets[current_frame as usize]);

        swapchain_framebuffer.get_image(0).transition_layout(ImageLayout::PresentSrc, Some(cmd), ash::vk::AccessFlags::empty(), ash::vk::AccessFlags::empty());
    }
}