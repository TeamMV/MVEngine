use std::mem;

use mvutils::unsafe_utils::Unsafe;
use shaderc::ShaderKind;

use mvcore::math::vec::{Vec2, Vec3, Vec4};
use mvcore::render::backend::Extent3D;
use mvcore::render::backend::buffer::{Buffer, BufferUsage, MemoryProperties, MVBufferCreateInfo};
use mvcore::render::backend::descriptor_set::{DescriptorPool, DescriptorPoolFlags, DescriptorPoolSize, DescriptorSet, DescriptorSetLayoutBinding, DescriptorType, MVDescriptorPoolCreateInfo, MVDescriptorSetCreateInfo};
use mvcore::render::backend::device::Device;
use mvcore::render::backend::framebuffer::{ClearColor, Framebuffer, MVFramebufferCreateInfo};
use mvcore::render::backend::image::{AccessFlags, ImageFormat, ImageLayout, ImageUsage};
use mvcore::render::backend::pipeline::{AttributeType, Compute, CullMode, Graphics, MVComputePipelineCreateInfo, MVGraphicsPipelineCreateInfo, Pipeline, Topology};
use mvcore::render::backend::sampler::{Filter, MipmapMode, MVSamplerCreateInfo, Sampler, SamplerAddressMode};
use mvcore::render::backend::shader::ShaderStage;
use mvcore::render::camera::OrthographicCamera;
use mvcore::render::mesh::Mesh;
use mvcore::render::renderer::Renderer;
use mvcore::render::window::Window;

#[repr(C)]
struct Vertex {
    position: Vec3,
    tex_coord: Vec2,
}

impl Vertex {
    pub fn get_attribute_description() -> Vec<AttributeType> {
        vec![AttributeType::Float32x3, AttributeType::Float32x2]
    }
}

#[repr(C)]
pub struct Transform {
    pub position: Vec4,
    pub rotation: Vec4,
    pub scale: Vec4,
}

static MAX_BATCH_SIZE: u32 = 10000;

pub struct Renderer2D {
    device: Device,
    core_renderer: Renderer,
    quad_mesh: Mesh,
    camera_sets: Vec<DescriptorSet>,
    camera_buffers: Vec<Buffer>,
    descriptor_pool: DescriptorPool,
    transforms: Vec<Transform>,
    transforms_buffers: Vec<Buffer>,
    transforms_sets: Vec<DescriptorSet>,
    main_pipeline: Pipeline::<Graphics>,
    tonemap_pipeline: Pipeline::<Compute>,
    tonemap_sets: Vec<DescriptorSet>,
    geometry_framebuffers: Vec<Framebuffer>,
    default_sampler: Sampler,
}

impl Renderer2D {
    pub fn new(device: Device, window: &Window) -> Self {
        let renderer = Renderer::new(window, device.clone());

        //
        // Pool
        //
        let descriptor_pool = DescriptorPool::new(device.clone(), MVDescriptorPoolCreateInfo {
            sizes: vec![
                DescriptorPoolSize{ ty: DescriptorType::CombinedImageSampler, count: 1000 },
                DescriptorPoolSize{ ty: DescriptorType::StorageImage, count: 1000 },
                DescriptorPoolSize{ ty: DescriptorType::UniformBuffer, count: 1000 },
                DescriptorPoolSize{ ty: DescriptorType::StorageBuffer, count: 1000 },
            ],
            max_sets: 1000,
            flags: DescriptorPoolFlags::FREE_DESCRIPTOR,
            label: Some("Main Descriptor Set".to_string()),
        });

        //
        // Camera Set
        //

        // Dummy Camera, we'll use ECS later on
        let camera = OrthographicCamera::new(renderer.get_swapchain().get_extent().width, renderer.get_swapchain().get_extent().height);

        let mut camera_buffers = Vec::new();

        for _ in 0..renderer.get_max_frames_in_flight() {
            let mut buffer = Buffer::new(device.clone(), MVBufferCreateInfo {
                instance_size: 128,
                instance_count: 1,
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_properties: MemoryProperties::HOST_VISIBLE | MemoryProperties::HOST_COHERENT,
                minimum_alignment: 1,
                memory_usage: gpu_alloc::UsageFlags::HOST_ACCESS,
                label: Some("Camera Uniform Buffer".to_string()),
            });

            let matrices = [camera.get_view(), camera.get_projection()];
            let bytes = unsafe { std::slice::from_raw_parts(matrices.as_ptr() as *const u8, 128)};

            buffer.write(bytes, 0, None);
            camera_buffers.push(buffer);
        }

        let mut camera_sets = Vec::new();

        for index in 0..renderer.get_max_frames_in_flight() {
            let mut camera_set = DescriptorSet::new(device.clone(), MVDescriptorSetCreateInfo {
                pool: descriptor_pool.clone(),
                bindings: vec![
                    DescriptorSetLayoutBinding{
                        index: 0,
                        stages: ShaderStage::Vertex,
                        ty: DescriptorType::UniformBuffer,
                        count: 1,
                    }
                ],
                label: Some("Camera Set".to_string()),
            });

            camera_set.add_buffer(0, &camera_buffers[index as usize], 0, 128);
            camera_set.build();

            camera_sets.push(camera_set);
        }

        //
        // Transforms
        //

        let mut transforms_buffers = Vec::new();
        for _ in 0..renderer.get_max_frames_in_flight() {
            transforms_buffers.push(Self::create_transform_buffer(device.clone()));
        }

        let mut transforms_sets = Vec::new();
        for index in 0..renderer.get_max_frames_in_flight() {
            let mut set = DescriptorSet::new(device.clone(), MVDescriptorSetCreateInfo {
                pool: descriptor_pool.clone(),
                bindings: vec![DescriptorSetLayoutBinding {
                    index: 0,
                    stages: ShaderStage::Vertex,
                    ty: DescriptorType::StorageBuffer,
                    count: 1,
                }],
                label: Some("Transforms set".to_string())
            });

            set.add_buffer(0, &transforms_buffers[index as usize], 0, transforms_buffers[index as usize].get_size());
            set.build();

            transforms_sets.push(set);
        }

        //
        // Mesh
        //

        let vertices = vec![
            Vertex { position: Vec3::new(-1.0, 1.0, 0.0), tex_coord: Vec2::default() }, // 0
            Vertex { position: Vec3::new(-1.0, -1.0, 0.0), tex_coord: Vec2::default() }, // 1
            Vertex { position: Vec3::new(1.0, -1.0, 0.0), tex_coord: Vec2::default() }, // 2
            Vertex { position: Vec3::new(1.0, 1.0, 0.0), tex_coord: Vec2::default() }, // 3
        ];

        let vertices_bytes = unsafe { std::slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices.len() * std::mem::size_of::<Vertex>()) };
        let indices = vec![
            0u32, 1, 2,
            0, 2, 3
        ];

        let quad_mesh = Mesh::new(device.clone(), vertices_bytes, Some(&indices), Some("Main Quad Vertex Buffer".to_string()));

        //
        // Framebuffer
        //
        let mut geometry_framebuffers = Vec::new();
        for _ in 0..renderer.get_max_frames_in_flight() {
            let framebuffer = Framebuffer::new(device.clone(), MVFramebufferCreateInfo{
                attachment_formats: vec![ImageFormat::R32G32B32A32, ImageFormat::D16], // TODO: 16 bits
                extent: renderer.get_swapchain().get_extent(),
                image_usage_flags: ImageUsage::empty(),
                render_pass_info: None,
                label: Some("Geometry Framebuffer".to_string()),
            });

            geometry_framebuffers.push(framebuffer);
        }

        //
        // Pipeline
        //

        // Shaders
        let vertex_shader = renderer.compile_shader(include_str!("shaders/default.vert"), ShaderKind::Vertex, Some("Default Quad Vertex Shader".to_string()), &[]);
        let fragment_shader = renderer.compile_shader(include_str!("shaders/default.frag"), ShaderKind::Fragment, Some("Default Quad Fragment Shader".to_string()), &[]);

        let default_pipeline = Pipeline::<Graphics>::new(device.clone(), MVGraphicsPipelineCreateInfo{
            shaders: vec![vertex_shader, fragment_shader],
            attributes: Vertex::get_attribute_description(),
            extent: renderer.get_swapchain().get_extent(),
            topology: Topology::Triangle,
            cull_mode: CullMode::Back,
            enable_depth_test: true,
            depth_clamp: false,
            blending_enable: true,
            descriptor_sets: vec![camera_sets[0].get_layout(), transforms_sets[0].get_layout()],
            push_constants: vec![],
            framebuffer: geometry_framebuffers[0].clone(),
            color_attachments_count: 1,
            label: Some("Default Quad Pipeline".to_string()),
        });

        //
        // Default sampler
        //

        let default_sampler = Sampler::new(device.clone(), MVSamplerCreateInfo{
            address_mode: SamplerAddressMode::ClampToEdge,
            filter_mode: Filter::Nearest,
            mipmap_mode: MipmapMode::Nearest,
            anisotropy: false,
            label: Some("Main Nearest Sampler".to_string()),
        });

        //
        // Tonemapping
        //
        let tonemap_shader = renderer.compile_shader(include_str!("shaders/tonemap.comp"), ShaderKind::Compute, Some("Tonemap Shader".to_string()), &[]);

        let mut tonemap_sets = Vec::new();
        for index in 0..renderer.get_max_frames_in_flight() {
            let mut set = DescriptorSet::new(device.clone(), MVDescriptorSetCreateInfo {
                pool: descriptor_pool.clone(),
                bindings: vec![DescriptorSetLayoutBinding {
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
                }],
                label: Some("Tonemap set".to_string())
            });

            set.add_image(0, &geometry_framebuffers[index as usize].get_image(0), &default_sampler, ImageLayout::ShaderReadOnlyOptimal);
            set.add_image(1, &renderer.get_swapchain().get_framebuffer(index as usize).get_image(0), &default_sampler, ImageLayout::General);
            set.build();

            tonemap_sets.push(set);
        }

        let tonemap_pipeline = Pipeline::<Compute>::new(device.clone(), MVComputePipelineCreateInfo {
            shader: tonemap_shader,
            descriptor_sets: vec![tonemap_sets[0].get_layout()],
            push_constants: vec![],
            label: Some("Tonemap Pipeline".to_string()),
        });

        Self {
            device,
            core_renderer: renderer,
            quad_mesh,
            transforms: vec![],
            camera_sets,
            camera_buffers,
            descriptor_pool,
            transforms_buffers,
            transforms_sets,
            main_pipeline: default_pipeline,
            tonemap_sets,
            geometry_framebuffers,
            default_sampler,
            tonemap_pipeline
        }
    }

    pub fn draw(&mut self) {
        let image_index = self.core_renderer.begin_frame().unwrap();
        let current_frame = self.core_renderer.get_current_frame_index();
        let cmd = unsafe { Unsafe::cast_static(self.core_renderer.get_current_command_buffer()) };

        let swapchain = self.core_renderer.get_swapchain_mut();
        let extent = swapchain.get_extent();
        let swapchain_framebuffer = &swapchain.get_current_framebuffer();
        let geometry_framebuffer = &self.geometry_framebuffers[current_frame as usize];

        // Push data to the storage buffer
        let bytes = unsafe { std::slice::from_raw_parts(self.transforms.as_ptr() as *const u8, self.transforms.len() * mem::size_of::<Transform>()) };
        self.transforms_buffers[current_frame as usize].write(bytes, 0, None);

        // GEOMETRY PASS
        geometry_framebuffer.begin_render_pass(cmd, &[ClearColor::Color([0.0, 0.0, 0.0, 1.0]), ClearColor::Depth { depth: 1.0, stencil: 0 }], swapchain.get_extent());

        self.main_pipeline.bind(cmd);

        self.camera_sets[current_frame as usize].bind(cmd, &self.main_pipeline, 0);
        self.transforms_sets[current_frame as usize].bind(cmd, &self.main_pipeline, 1);

        self.quad_mesh.draw_instanced(cmd, 0, self.transforms.len() as u32);

        geometry_framebuffer.end_render_pass(cmd);

        // TONEMAP PASS
        geometry_framebuffer.get_image(0).transition_layout(ImageLayout::ShaderReadOnlyOptimal, Some(&cmd), AccessFlags::empty(), AccessFlags::empty());
        swapchain_framebuffer.get_image(0).transition_layout(ImageLayout::General, Some(&cmd), AccessFlags::empty(), AccessFlags::empty());

        self.tonemap_pipeline.bind(cmd);
        self.tonemap_sets[current_frame as usize].bind(cmd, &self.tonemap_pipeline, 0);
        cmd.dispatch(Extent3D{
            width: geometry_framebuffer.get_image(0).get_extent().width / 8 + 1,
            height: geometry_framebuffer.get_image(0).get_extent().height / 8 + 1,
            depth: 1,
        });

        swapchain_framebuffer.get_image(0).transition_layout(ImageLayout::PresentSrc, Some(&cmd), AccessFlags::empty(), AccessFlags::empty());
        self.core_renderer.end_frame().unwrap();

        // Clear all data
        self.transforms.clear();
    }

    pub fn add_quad(&mut self, transform: Transform) {
        if self.transforms.len() as u32 > MAX_BATCH_SIZE {
            log::error!("Todo: multiple batches");
            panic!();
        }
        self.transforms.push(transform);
    }

    fn create_transform_buffer(device: Device) -> Buffer {
        Buffer::new(device.clone(), MVBufferCreateInfo {
            instance_size: 100 * 100 * (mem::size_of::<Transform>() as u64),
            instance_count: 1,
            buffer_usage: BufferUsage::STORAGE_BUFFER,
            memory_properties: MemoryProperties::DEVICE_LOCAL,
            minimum_alignment: 1,
            memory_usage: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
            label: Some("Matrix Storage Buffer".to_string()),
        })
    }
}