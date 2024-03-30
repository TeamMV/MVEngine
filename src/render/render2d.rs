use crate::render::backend::buffer::{Buffer, BufferUsage, MVBufferCreateInfo, MemoryProperties};
use crate::render::backend::command_buffer::CommandBuffer;
use crate::render::backend::descriptor_set::{
    DescriptorPool, DescriptorPoolFlags, DescriptorPoolSize, DescriptorSet, DescriptorSetLayout,
    DescriptorSetLayoutBinding, DescriptorType, MVDescriptorPoolCreateInfo,
    MVDescriptorSetCreateInfo, MVDescriptorSetFromLayoutCreateInfo,
    MVDescriptorSetLayoutCreateInfo,
};
use crate::render::backend::device::Device;
use crate::render::backend::framebuffer::{ClearColor, Framebuffer, MVFramebufferCreateInfo};
use crate::render::backend::image::{
    Image, ImageAspect, ImageFormat, ImageLayout, ImageTiling, ImageType, ImageUsage,
    MVImageCreateInfo,
};
use crate::render::backend::pipeline::{
    AttributeType, Compute, CullMode, Graphics, MVComputePipelineCreateInfo,
    MVGraphicsPipelineCreateInfo, Pipeline, Topology,
};
use crate::render::backend::sampler::{
    Filter, MVSamplerCreateInfo, MipmapMode, Sampler, SamplerAddressMode,
};
use crate::render::backend::shader::{MVShaderCreateInfo, Shader, ShaderStage};
use crate::render::backend::{Extent2D, Extent3D};
use crate::render::state::State;
use crate::render::window::WindowCreateInfo;
use bitflags::Flags;
use glam::Mat4;
use gpu_alloc::UsageFlags;
use hashbrown::HashMap;
use shaderc::{OptimizationLevel, ShaderKind, TargetEnv};
use std::sync::Arc;

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
    fn new(
        state: &State,
        info: &WindowCreateInfo,
        matrix_layout: DescriptorSetLayout,
        vertex: Shader,
        geometry: Option<Shader>,
        fragment: Shader,
        attributes: Vec<AttributeType>,
        framebuffer: Framebuffer,
        #[cfg(debug_assertions)] label: &'static str,
    ) -> Self {
        let label = label.to_string();

        let mut shaders = vec![vertex.clone(), fragment.clone()];
        if let Some(geometry) = geometry.clone() {
            shaders.push(geometry);
        }

        let pipeline = Pipeline::<Graphics>::new(
            state.get_device(),
            MVGraphicsPipelineCreateInfo {
                shaders,
                attributes: attributes.clone(),
                extent: Extent2D {
                    width: info.width,
                    height: info.height,
                },
                topology: geometry
                    .is_some()
                    .then_some(Topology::Point)
                    .unwrap_or(Topology::Triangle),
                cull_mode: CullMode::Back,
                enable_depth_test: false,
                depth_clamp: false,
                blending_enable: true,
                descriptor_sets: vec![matrix_layout.clone()],
                push_constants: vec![],
                framebuffer: framebuffer.clone(),
                color_attachments_count: 1,
                label: Some(label.clone()),
            },
        );

        GraphicsPipeline2d {
            vertex,
            geometry,
            fragment,
            attributes,
            matrix_layout,
            pipeline,
            label,
            framebuffer: framebuffer.clone(),
        }
    }

    fn resize(&mut self, state: &State) {
        let mut shaders = vec![self.vertex.clone(), self.fragment.clone()];
        if let Some(geometry) = self.geometry.clone() {
            shaders.push(geometry);
        }

        self.pipeline = Pipeline::<Graphics>::new(
            state.get_device(),
            MVGraphicsPipelineCreateInfo {
                shaders,
                attributes: self.attributes.clone(),
                extent: state.get_swapchain().get_extent(),
                topology: self
                    .geometry
                    .is_some()
                    .then_some(Topology::Point)
                    .unwrap_or(Topology::Triangle),
                cull_mode: CullMode::Back,
                enable_depth_test: false,
                depth_clamp: false,
                blending_enable: true,
                descriptor_sets: vec![self.matrix_layout.clone()],
                push_constants: vec![],
                framebuffer: self.framebuffer.clone(),
                color_attachments_count: 1,
                label: Some(self.label.clone()),
            },
        );
    }
}

pub struct Effect2d {
    pipelines: Vec<EffectPipeline2d>,
    sampler: Arc<Sampler>,
    has_bound: bool,
}

impl Effect2d {
    pub(crate) fn new(
        state: &State,
        shader: Shader,
        pool: &DescriptorPool,
        sampler: Arc<Sampler>,
        textures: Vec<&[Image]>,
    ) -> Self {
        debug_assert!(
            textures.is_empty()
                || textures.len() == state.get_swapchain().get_max_frames_in_flight() as usize
        );

        let effect_set_layout = DescriptorSetLayout::new(
            state.get_device(),
            MVDescriptorSetLayoutCreateInfo {
                bindings: vec![DescriptorSetLayoutBinding {
                    index: 0,
                    stages: ShaderStage::Compute,
                    ty: DescriptorType::UniformBuffer,
                    count: 1,
                }],
                label: Some("Effect set layout 2d".to_string()),
            },
        );

        let mut pipelines = Vec::new();
        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            pipelines.push(EffectPipeline2d::new(
                state,
                effect_set_layout.clone(),
                shader.clone(),
                pool,
                &sampler,
                if textures.is_empty() {
                    &[]
                } else {
                    textures[index as usize]
                },
            ));
        }

        Self {
            pipelines,
            sampler,
            has_bound: false,
        }
    }

    pub fn update_textures(&mut self, textures: Vec<&[Image]>) {
        debug_assert_eq!(textures.len(), self.pipelines.len());

        for (index, pipeline) in self.pipelines.iter_mut().enumerate() {
            pipeline.update_additional_textures(&self.sampler, textures[index]);
        }
    }

    fn get(&mut self, frame: u32) -> &mut EffectPipeline2d {
        &mut self.pipelines[frame as usize]
    }

    fn bind(&mut self, input_images: Vec<Image>, output_images: Vec<Image>) {
        for (index, pipeline) in self.pipelines.iter_mut().enumerate() {
            pipeline.bind(
                &self.sampler,
                input_images[index].clone(),
                output_images[index].clone(),
                self.has_bound,
            );
        }
        self.has_bound = true;
    }

    fn unbind(&mut self) {
        for pipeline in &mut self.pipelines {
            pipeline.unbind();
        }
    }

    fn get_input_output_images(&mut self) -> Vec<(Option<Image>, Option<Image>)> {
        self.pipelines
            .iter_mut()
            .map(|pipeline| (pipeline.input_image.take(), pipeline.output_image.take()))
            .collect()
    }

    fn set_input_output_images(&mut self, images: Vec<(Option<Image>, Option<Image>)>) {
        for (index, (input, output)) in images.into_iter().enumerate() {
            self.pipelines[index].update_images(&self.sampler, input.unwrap(), output.unwrap());
        }
    }

    fn set_output_images(&mut self, images: Vec<Image>) {
        for (index, pipeline) in self.pipelines.iter_mut().enumerate() {
            pipeline.set_output_image(&self.sampler, images[index].clone());
        }
    }
}

struct EffectPipeline2d {
    shader: Shader,

    image_set: DescriptorSet,
    input_image: Option<Image>,
    output_image: Option<Image>,

    pipeline: Pipeline<Compute>,
}

impl EffectPipeline2d {
    fn new(
        state: &State,
        effect_set_layout: DescriptorSetLayout,
        shader: Shader,
        pool: &DescriptorPool,
        sampler: &Sampler,
        input_textures: &[Image],
    ) -> Self {
        let mut bindings = vec![
            DescriptorSetLayoutBinding {
                index: 0,
                stages: ShaderStage::Compute,
                ty: DescriptorType::CombinedImageSampler,
                count: 1,
            },
            DescriptorSetLayoutBinding {
                index: 1,
                stages: ShaderStage::Compute,
                ty: DescriptorType::StorageImage,
                count: 1,
            },
        ];

        for index in 0..input_textures.len() {
            bindings.push(DescriptorSetLayoutBinding {
                index: (index + 2) as u32,
                stages: ShaderStage::Compute,
                ty: DescriptorType::CombinedImageSampler,
                count: 1,
            });
        }

        let image_set_layout = DescriptorSetLayout::new(
            state.get_device().clone(),
            MVDescriptorSetLayoutCreateInfo {
                bindings,
                label: Some("Effect image set layout 2d".to_string()),
            },
        );

        let mut image_set = DescriptorSet::from_layout(
            state.get_device(),
            MVDescriptorSetFromLayoutCreateInfo {
                pool: pool.clone(),
                layout: image_set_layout.clone(),
                label: Some("Image Set".to_string()),
            },
        );

        for index in 0..input_textures.len() {
            image_set.add_image(
                (index + 2) as u32,
                &input_textures[index],
                sampler,
                ImageLayout::ShaderReadOnlyOptimal,
            );
        }

        let pipeline = Pipeline::<Compute>::new(
            state.get_device(),
            MVComputePipelineCreateInfo {
                shader: shader.clone(),
                descriptor_sets: vec![image_set_layout.clone(), effect_set_layout.clone()],
                push_constants: vec![],
                label: Some("Effect pipeline 2d".to_string()),
            },
        );

        Self {
            shader,
            image_set,
            pipeline,
            input_image: None,
            output_image: None,
        }
    }

    fn bind(&mut self, sampler: &Sampler, input_image: Image, output_image: Image, build: bool) {
        self.input_image = Some(input_image.clone());
        self.output_image = Some(output_image.clone());

        if !build {
            self.image_set
                .add_image(0, &input_image, sampler, ImageLayout::ShaderReadOnlyOptimal);
            self.image_set
                .add_image(1, &output_image, sampler, ImageLayout::General);
            self.image_set.build();
        } else {
            self.image_set.update_image(
                0,
                &input_image,
                sampler,
                ImageLayout::ShaderReadOnlyOptimal,
            );
            self.image_set
                .update_image(1, &output_image, sampler, ImageLayout::General);
        }
    }

    fn unbind(&mut self) {
        self.input_image = None;
        self.output_image = None;
    }

    fn run(&mut self, cmd: &CommandBuffer, data_set: &mut DescriptorSet) {
        let input = self
            .input_image
            .as_mut()
            .expect("Input image cannot be None when running effect shader");
        let output = self
            .output_image
            .as_mut()
            .expect("Input image cannot be None when running effect shader");
        input.transition_layout(
            ImageLayout::ShaderReadOnlyOptimal,
            Some(cmd),
            ash::vk::AccessFlags::empty(),
            ash::vk::AccessFlags::empty(),
        );
        output.transition_layout(
            ImageLayout::General,
            Some(cmd),
            ash::vk::AccessFlags::empty(),
            ash::vk::AccessFlags::empty(),
        );

        self.pipeline.bind(cmd);

        self.image_set.bind(cmd, &self.pipeline, 0);
        data_set.bind(cmd, &self.pipeline, 1);

        cmd.dispatch(Extent3D {
            width: input.get_extent().width / 8 + 1,
            height: input.get_extent().height / 8 + 1,
            depth: 1,
        });
    }

    fn update_images(&mut self, sampler: &Sampler, input_image: Image, output_image: Image) {
        self.input_image = Some(input_image.clone());
        self.output_image = Some(output_image.clone());

        self.image_set
            .update_image(0, &input_image, sampler, ImageLayout::ShaderReadOnlyOptimal);
        self.image_set
            .update_image(1, &output_image, sampler, ImageLayout::General);
    }

    fn set_output_image(&mut self, sampler: &Sampler, image: Image) {
        self.output_image = Some(image.clone());

        self.image_set
            .update_image(1, &image, sampler, ImageLayout::General);
    }

    fn set_input_image(&mut self, sampler: &Sampler, image: Image) {
        self.input_image = Some(image.clone());

        self.image_set
            .update_image(0, &image, sampler, ImageLayout::ShaderReadOnlyOptimal);
    }

    fn update_additional_textures(&mut self, sampler: &Sampler, input_textures: &[Image]) {
        for index in 0..input_textures.len() {
            self.image_set.update_image(
                (index + 2) as u32,
                &input_textures[index],
                sampler,
                ImageLayout::ShaderReadOnlyOptimal,
            );
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
    fn new(
        state: &State,
        sampler: &Sampler,
        pool: DescriptorPool,
        shader: Shader,
        input_image: Image,
        output_image: Image,
    ) -> Self {
        let image_set_layout = DescriptorSetLayout::new(
            state.get_device().clone(),
            MVDescriptorSetLayoutCreateInfo {
                bindings: vec![
                    DescriptorSetLayoutBinding {
                        index: 0,
                        stages: ShaderStage::Compute,
                        ty: DescriptorType::CombinedImageSampler,
                        count: 1,
                    },
                    DescriptorSetLayoutBinding {
                        index: 1,
                        stages: ShaderStage::Compute,
                        ty: DescriptorType::StorageImage,
                        count: 1,
                    },
                ],
                label: Some("Tone map pipeline image set 2d".to_string()),
            },
        );

        let mut image_set = DescriptorSet::from_layout(
            state.get_device(),
            MVDescriptorSetFromLayoutCreateInfo {
                pool,
                layout: image_set_layout.clone(),
                label: Some("Tone map set 2d".to_string()),
            },
        );

        image_set.add_image(0, &input_image, sampler, ImageLayout::ShaderReadOnlyOptimal);
        image_set.add_image(1, &output_image, sampler, ImageLayout::General);

        image_set.build();

        let pipeline = Pipeline::<Compute>::new(
            state.get_device(),
            MVComputePipelineCreateInfo {
                shader: shader.clone(),
                descriptor_sets: vec![image_set_layout],
                push_constants: vec![],
                label: Some("Tone map pipeline 2d".to_string()),
            },
        );

        Self {
            shader,
            image_set,
            pipeline,
            input_image,
            output_image,
        }
    }

    fn update_images(&mut self, sampler: &Sampler, input_image: &Image, output_image: &Image) {
        self.image_set
            .update_image(0, input_image, sampler, ImageLayout::ShaderReadOnlyOptimal);
        self.image_set
            .update_image(1, output_image, sampler, ImageLayout::General);

        self.input_image = input_image.clone();
        self.output_image = output_image.clone();
    }

    fn set_output(&mut self, sampler: &Sampler, image: Image) {
        self.image_set
            .update_image(1, &image, sampler, ImageLayout::General);

        self.output_image = image.clone();
    }

    fn set_input(&mut self, sampler: &Sampler, image: Image) {
        self.image_set
            .update_image(0, &image, sampler, ImageLayout::ShaderReadOnlyOptimal);

        self.input_image = image.clone();
    }

    fn run(&mut self, cmd: &CommandBuffer) {
        self.input_image.transition_layout(
            ImageLayout::ShaderReadOnlyOptimal,
            Some(cmd),
            ash::vk::AccessFlags::empty(),
            ash::vk::AccessFlags::empty(),
        );
        self.output_image.transition_layout(
            ImageLayout::General,
            Some(cmd),
            ash::vk::AccessFlags::empty(),
            ash::vk::AccessFlags::empty(),
        );

        self.pipeline.bind(cmd);

        self.image_set.bind(cmd, &self.pipeline, 0);

        cmd.dispatch(Extent3D {
            width: self.input_image.get_extent().width,
            height: self.input_image.get_extent().height,
            depth: 1,
        });
    }
}

struct Bloom {
    shader: Shader,
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
    sampler: Arc<Sampler>,

    square_pipeline: GraphicsPipeline2d,
    default_pipeline: GraphicsPipeline2d,

    tone_maps: Vec<ToneMap>,
    blooms: Vec<Bloom>,
    bloom: bool,

    available_effects: HashMap<String, Effect2d>,

    effect_images: Vec<Vec<Image>>,
    effects: Vec<String>,
    effect_uniform: Uniform,
    geometry_framebuffers: Vec<Framebuffer>,

    swapchain_images: Vec<Image>,
}

macro_rules! shader {
    ($state: ident, $compile: ident, $file: literal, $ty: ident) => {
        Shader::new(
            $state.get_device(),
            MVShaderCreateInfo {
                stage: ShaderStage::$ty,
                code: $compile(include_str!($file), $file, ShaderKind::$ty),
                label: Some($file.to_string()),
            },
        )
    };
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
                .compile_into_spirv(str, kind, name, "main", Some(&options))
                .unwrap();
            binary_result.as_binary().to_vec()
        }

        let square_vertex = shader!(state, compile, "shaders/2d/square.vert", Vertex);
        let square_geometry = shader!(state, compile, "shaders/2d/square.geom", Geometry);
        let vertex = shader!(state, compile, "shaders/2d/default.vert", Vertex);
        let fragment = shader!(state, compile, "shaders/2d/default.frag", Fragment);
        let invert = shader!(state, compile, "shaders/2d/invert.comp", Compute);
        let tone_map = shader!(state, compile, "shaders/2d/tonemap.comp", Compute);

        let descriptor_pool = DescriptorPool::new(
            state.get_device(),
            MVDescriptorPoolCreateInfo {
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
                    },
                ],
                max_sets: 1000,
                flags: DescriptorPoolFlags::FREE_DESCRIPTOR,
                label: Some("Descriptor pool 2d".to_string()),
            },
        );

        let sampler = Arc::new(Sampler::new(
            state.get_device(),
            MVSamplerCreateInfo {
                address_mode: SamplerAddressMode::ClampToEdge,
                filter_mode: Filter::Linear,
                mipmap_mode: MipmapMode::Linear,
                label: Some("Sampler 2d".to_string()),
            },
        ));

        let matrix_set_layout = DescriptorSetLayout::new(
            state.get_device(),
            MVDescriptorSetLayoutCreateInfo {
                bindings: vec![DescriptorSetLayoutBinding {
                    index: 0,
                    stages: ShaderStage::Vertex | ShaderStage::Geometry,
                    ty: DescriptorType::UniformBuffer,
                    count: 1,
                }],
                label: Some("Matrix descriptor set layout 2d".to_string()),
            },
        );
        let mut geometry_framebuffers = Vec::new();
        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            geometry_framebuffers.push(Framebuffer::new(
                state.get_device().clone(),
                MVFramebufferCreateInfo {
                    attachment_formats: vec![ImageFormat::R32G32B32A32],
                    extent: state.get_swapchain().get_extent(),
                    image_usage_flags: ImageUsage::SAMPLED,
                    render_pass_info: None,
                    label: Some("Geometry HDR framebuffer".to_string()),
                },
            ));
        }

        let square_pipeline = GraphicsPipeline2d::new(
            state,
            info,
            matrix_set_layout.clone(),
            square_vertex,
            Some(square_geometry),
            fragment.clone(),
            vec![
                AttributeType::Float32x2,
                AttributeType::Float32x2,
                AttributeType::Float32x4,
                AttributeType::Float32,
                AttributeType::Float32x4,
            ],
            geometry_framebuffers[0].clone(),
            "Square pipeline 2d",
        );

        let default_pipeline = GraphicsPipeline2d::new(
            state,
            info,
            matrix_set_layout.clone(),
            vertex,
            None,
            fragment,
            vec![
                AttributeType::Float32x2,
                AttributeType::Float32x2,
                AttributeType::Float32x4,
                AttributeType::Float32,
            ],
            geometry_framebuffers[0].clone(),
            "Default pipeline 2d",
        );

        let mut vertex_buffer = Buffer::new(
            state.get_device().clone(),
            MVBufferCreateInfo {
                instance_size: (2 + 2 + 4 + 1 + 4) * 2 * 4,
                instance_count: 1,
                buffer_usage: BufferUsage::VERTEX_BUFFER,
                memory_properties: MemoryProperties::DEVICE_LOCAL,
                minimum_alignment: 1,
                memory_usage: UsageFlags::FAST_DEVICE_ACCESS,
                label: Some("Vertex buffer 2d".to_string()),
            },
        );

        let mut matrix_buffers = Vec::new();
        let mut matrix_sets = Vec::new();

        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            matrix_buffers.push(Buffer::new(
                state.get_device(),
                MVBufferCreateInfo {
                    instance_size: 128,
                    instance_count: 1,
                    buffer_usage: BufferUsage::UNIFORM_BUFFER,
                    memory_properties: MemoryProperties::HOST_VISIBLE
                        | MemoryProperties::HOST_COHERENT,
                    minimum_alignment: 1,
                    memory_usage: UsageFlags::HOST_ACCESS,
                    label: Some("Matrix buffer 2d".to_string()),
                },
            ));
        }

        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            matrix_sets.push(DescriptorSet::from_layout(
                state.get_device(),
                MVDescriptorSetFromLayoutCreateInfo {
                    pool: descriptor_pool.clone(),
                    layout: matrix_set_layout.clone(),
                    label: Some("Matrix descriptor set 2d".to_string()),
                },
            ));
            matrix_sets[index as usize].add_buffer(
                0,
                &matrix_buffers[index as usize],
                0,
                matrix_buffers[index as usize].get_size(),
            );
            matrix_sets[index as usize].build();
        }

        let vertex_data = [
            100.0f32, -100.0, 150.0, 150.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0,
            0.0, // Left Square
            500.5, -100.0, 150.0, 150.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0,
            0.0, // Right Square
        ];

        let byte_data_vertex = unsafe {
            std::slice::from_raw_parts(vertex_data.as_ptr() as *const u8, vertex_data.len() * 4)
        };

        vertex_buffer.write(byte_data_vertex, 0, None);

        let effect_set_layout = DescriptorSetLayout::new(
            state.get_device(),
            MVDescriptorSetLayoutCreateInfo {
                bindings: vec![DescriptorSetLayoutBinding {
                    index: 0,
                    stages: ShaderStage::Compute,
                    ty: DescriptorType::UniformBuffer,
                    count: 1,
                }],
                label: Some("Effect set layout 2d".to_string()),
            },
        );

        let mut effect_sets = Vec::new();
        let mut effect_buffers = Vec::new();

        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            effect_buffers.push(Buffer::new(
                state.get_device(),
                MVBufferCreateInfo {
                    instance_size: 4,
                    instance_count: 1,
                    buffer_usage: BufferUsage::UNIFORM_BUFFER,
                    memory_properties: MemoryProperties::HOST_VISIBLE
                        | MemoryProperties::HOST_COHERENT,
                    minimum_alignment: 1,
                    memory_usage: UsageFlags::HOST_ACCESS,
                    label: Some("Effect buffer 2d".to_string()),
                },
            ));
        }

        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            effect_sets.push(DescriptorSet::from_layout(
                state.get_device(),
                MVDescriptorSetFromLayoutCreateInfo {
                    pool: descriptor_pool.clone(),
                    layout: effect_set_layout.clone(),
                    label: Some("Effect descriptor set 2d".to_string()),
                },
            ));
            effect_sets[index as usize].add_buffer(
                0,
                &effect_buffers[index as usize],
                0,
                effect_buffers[index as usize].get_size(),
            );
            effect_sets[index as usize].build();
        }

        let mut effects_images: Vec<Vec<Image>> = Vec::new();

        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            let mut image_pair = Vec::new();

            for index in 0..2 {
                image_pair.push(Image::new(
                    state.get_device().clone(),
                    MVImageCreateInfo {
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
                    },
                ));
            }
            effects_images.push(image_pair);
        }

        let mut tone_maps = Vec::new();
        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            tone_maps.push(ToneMap::new(
                state,
                &sampler,
                descriptor_pool.clone(),
                tone_map.clone(),
                geometry_framebuffers[index as usize].get_image(0).clone(),
                state
                    .get_swapchain()
                    .get_framebuffer(index as usize)
                    .get_image(0)
                    .clone(),
            ));
        }

        let framebuffer_images = state
            .get_swapchain()
            .get_framebuffers()
            .into_iter()
            .map(|framebuffer| framebuffer.get_image(0))
            .collect();

        let mut available_effects = HashMap::new();

        available_effects.insert(
            "invert".to_string(),
            Effect2d::new(state, invert, &descriptor_pool, sampler.clone(), vec![]),
        );

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
            effects: Vec::new(),
            effect_uniform: Uniform {
                layout: effect_set_layout,
                sets: effect_sets,
                buffers: effect_buffers,
            },
            sampler,
            tone_maps,
            geometry_framebuffers,
            effect_images: effects_images,
            available_effects,
            swapchain_images: framebuffer_images,
            blooms: vec![],
            bloom: false,
        }
    }

    pub(crate) fn resize(&mut self, state: &State) {
        state.get_device().wait_idle();

        self.swapchain_images = state
            .get_swapchain()
            .get_framebuffers()
            .into_iter()
            .map(|framebuffer| framebuffer.get_image(0))
            .collect();

        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            self.geometry_framebuffers[index as usize] = Framebuffer::new(
                state.get_device().clone(),
                MVFramebufferCreateInfo {
                    attachment_formats: vec![ImageFormat::R32G32B32A32],
                    extent: state.get_swapchain().get_extent(),
                    image_usage_flags: ImageUsage::SAMPLED,
                    render_pass_info: None,
                    label: Some("Geometry HDR framebuffer".to_string()),
                },
            );
        }

        self.effect_images.clear();
        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            let mut image_pair = Vec::new();
            for index in 0..2 {
                image_pair.push(Image::new(
                    state.get_device().clone(),
                    MVImageCreateInfo {
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
                    },
                ));
            }
            self.effect_images.push(image_pair);
        }

        self.square_pipeline.resize(state);
        self.default_pipeline.resize(state);

        let effects = self.effects.drain(..).collect::<Vec<_>>();

        for index in 0..state.get_swapchain().get_max_frames_in_flight() {
            self.tone_maps[index as usize].update_images(
                &self.sampler,
                &self.geometry_framebuffers[index as usize]
                    .get_image(0)
                    .clone(),
                &state
                    .get_swapchain()
                    .get_framebuffer(index as usize)
                    .get_image(0),
            );
        }

        for effect in effects {
            self.enable_effect(&effect);
        }
    }

    pub(crate) fn get_effect_list(&self) -> Vec<String> {
        self.effects.clone()
    }

    pub(crate) fn get_effect(&mut self, name: &str) -> &mut Effect2d {
        self.available_effects.get_mut(&name.to_string()).unwrap()
    }

    pub(crate) fn update_matrices(
        &mut self,
        state: &State,
        cmd: &CommandBuffer,
        view: Mat4,
        proj: Mat4,
    ) {
        let buffer = &mut self.matrix_uniform.buffers[state.get_current_frame_index() as usize];
        let matrices = [view, proj];
        let bytes = unsafe { std::slice::from_raw_parts(matrices.as_ptr() as *const u8, 128) };
        buffer.write(bytes, 0, Some(cmd));
    }

    pub(crate) fn enable_effect(&mut self, effect: &str) {
        if self.effects.contains(&effect.to_string()) { return; }
        let mut first = Vec::new();
        let mut second = Vec::new();
        for frame_index in 0..self.tone_maps.len() {
            first.push(self.effect_images[0][frame_index].clone());
            second.push(self.effect_images[1][frame_index].clone());
        }
        if self.effects.is_empty() {
            for (index, tonemap) in self.tone_maps.iter_mut().enumerate() {
                tonemap.set_output(&self.sampler, first[index].clone());
            }
            self.available_effects
                .get_mut(&effect.to_string())
                .unwrap()
                .bind(first, self.swapchain_images.clone());
        } else if self.effects.len() % 2 == 0 {
            self.available_effects
                .get_mut(self.effects.last().unwrap())
                .unwrap()
                .set_output_images(first.clone());
            self.available_effects
                .get_mut(&effect.to_string())
                .unwrap()
                .bind(first, self.swapchain_images.clone());
        } else {
            self.available_effects
                .get_mut(self.effects.last().unwrap())
                .unwrap()
                .set_output_images(second.clone());
            self.available_effects
                .get_mut(&effect.to_string())
                .unwrap()
                .bind(second, self.swapchain_images.clone());
        }

        self.effects.push(effect.to_string());
    }

    pub(crate) fn remove_effect_by_index(&mut self, index: usize) {
        if index >= self.effects.len() { return; }
        for i in index..self.effects.len() - 1 {
            self.swap_effects_by_index(i, i + 1);
        }
        self.pop_effect();
    }

    pub(crate) fn remove_effect(&mut self, effect: &str) {
        if let Some((index, _)) = self
            .effects
            .iter()
            .enumerate()
            .find(|(_, e)| e.as_str() == effect)
        {
            self.remove_effect_by_index(index);
        }
    }

    pub(crate) fn pop_effect(&mut self) {
        if let Some(effect) = self.effects.pop() {
            self.available_effects.get_mut(&effect).unwrap().unbind();
            if let Some(effect) = self.effects.last() {
                self.available_effects
                    .get_mut(effect)
                    .unwrap()
                    .set_output_images(self.swapchain_images.clone());
            } else {
                for (index, tonemap) in self.tone_maps.iter_mut().enumerate() {
                    tonemap.set_output(&self.sampler, self.swapchain_images[index].clone());
                }
            }
        }
    }

    pub(crate) fn swap_effects(&mut self, a: &str, b: &str) {
        if a == b {
            return;
        }
        if let Some((a, _)) = self
            .effects
            .iter()
            .enumerate()
            .find(|(_, e)| e.as_str() == a)
        {
            if let Some((b, _)) = self
                .effects
                .iter()
                .enumerate()
                .find(|(_, e)| e.as_str() == b)
            {
                self.swap_effects_by_index(a, b);
            }
        }
    }

    pub(crate) fn swap_effects_by_index(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        if a > self.effects.len() || b > self.effects.len() {
            log::error!(
                "Couldn't swap effects {a} and {b}: Out of bounds for length {}",
                self.effects.len()
            );
            return;
        }
        let a_images = self
            .available_effects
            .get_mut(&self.effects[a])
            .unwrap()
            .get_input_output_images();
        let b_images = self
            .available_effects
            .get_mut(&self.effects[b])
            .unwrap()
            .get_input_output_images();
        self.available_effects
            .get_mut(&self.effects[a])
            .unwrap()
            .set_input_output_images(b_images);
        self.available_effects
            .get_mut(&self.effects[b])
            .unwrap()
            .set_input_output_images(a_images);
        self.effects.swap(a, b);
    }

    pub(crate) fn enable_bloom(&mut self) {
        if self.bloom { return; }
        self.bloom = true;
    }

    pub(crate) fn disable_bloom(&mut self) {
        if !self.bloom { return; }
        self.bloom = false;
        for (index, tonemap) in self.tone_maps.iter_mut().enumerate() {
            tonemap.set_output(&self.sampler, self.geometry_framebuffers[index].get_image(0));
        }
    }

    pub(crate) fn draw(&mut self, state: &State, cmd: &CommandBuffer) {
        let swapchain = state.get_swapchain();
        let swapchain_framebuffer = swapchain.get_current_framebuffer();
        let matrix_set = &mut self.matrix_uniform.sets[swapchain.get_current_frame() as usize];
        let current_frame = swapchain.get_current_frame();
        let geometry_framebuffer = &self.geometry_framebuffers[current_frame as usize];

        geometry_framebuffer.begin_render_pass(
            cmd,
            &[ClearColor::Color([0.0, 0.0, 0.0, 1.0])],
            swapchain.get_extent(),
        );

        self.square_pipeline.pipeline.bind(cmd);

        cmd.bind_vertex_buffer(&self.vertex_buffer);

        matrix_set.bind(&cmd, &self.square_pipeline.pipeline, 0);

        cmd.draw(2, 0);

        geometry_framebuffer.end_render_pass(cmd);

        self.tone_maps[current_frame as usize].run(cmd);

        for effect in &self.effects {
            self.available_effects
                .get_mut(effect)
                .unwrap()
                .get(current_frame)
                .run(cmd, &mut self.effect_uniform.sets[current_frame as usize]);
        }

        swapchain_framebuffer.get_image(0).transition_layout(
            ImageLayout::PresentSrc,
            Some(cmd),
            ash::vk::AccessFlags::empty(),
            ash::vk::AccessFlags::empty(),
        );
    }
}
