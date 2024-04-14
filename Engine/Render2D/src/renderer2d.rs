use std::mem;
use std::sync::Arc;

use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use shaderc::ShaderKind;
use mvcore::asset::asset::AssetType;
use mvcore::asset::manager::{AssetHandle, AssetManager};

use mvcore::math::vec::{Vec2, Vec3, Vec4};
use mvcore::render::backend::buffer::{Buffer, BufferUsage, MVBufferCreateInfo, MemoryProperties};
use mvcore::render::backend::descriptor_set::{
    DescriptorPool, DescriptorPoolFlags, DescriptorPoolSize, DescriptorSet,
    DescriptorSetLayoutBinding, DescriptorType, MVDescriptorPoolCreateInfo,
    MVDescriptorSetCreateInfo,
};
use mvcore::render::backend::device::Device;
use mvcore::render::backend::framebuffer::{ClearColor, Framebuffer, MVFramebufferCreateInfo};
use mvcore::render::backend::image::{AccessFlags, Image, ImageAspect, ImageFormat, ImageLayout, ImageTiling, ImageType, ImageUsage, MVImageCreateInfo};
use mvcore::render::backend::pipeline::{
    AttributeType, Compute, CullMode, Graphics, MVComputePipelineCreateInfo,
    MVGraphicsPipelineCreateInfo, Pipeline, Topology,
};
use mvcore::render::backend::sampler::{
    Filter, MVSamplerCreateInfo, MipmapMode, Sampler, SamplerAddressMode,
};
use mvcore::render::backend::shader::ShaderStage;
use mvcore::render::backend::{Extent2D, Extent3D};
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
    pub position: Vec3,
    pub rotation: f32,
    pub scale: Vec2,
}

static MAX_BATCH_SIZE: u32 = 10000;

pub struct Renderer2D {
    device: Device,
    core_renderer: Arc<DangerousCell<Renderer>>,
    quad_mesh: Mesh,
    camera_sets: Vec<DescriptorSet>,
    camera_buffers: Vec<Buffer>,
    descriptor_pool: DescriptorPool,
    transforms: Vec<Transform>,
    transforms_buffers: Vec<Buffer>,
    transforms_sets: Vec<DescriptorSet>,
    main_pipeline: Pipeline<Graphics>,
    geometry_framebuffers: Vec<Framebuffer>,
    extent: Extent2D,
    manager: Arc<AssetManager>,
    handle: AssetHandle,
    default_sampler: Sampler,
    atlas_sets: Vec<DescriptorSet>,
    default_image: Image
}

impl Renderer2D {

    pub fn get_geometry_image(&self, frame_index: usize) -> Image {
        self.geometry_framebuffers[frame_index].get_image(0).clone()
    }

    pub fn new(device: Device, renderer: Arc<DangerousCell<Renderer>>, extent: Extent2D) -> Self {

        //
        // Pool
        //
        let descriptor_pool = DescriptorPool::new(
            device.clone(),
            MVDescriptorPoolCreateInfo {
                sizes: vec![
                    DescriptorPoolSize {
                        ty: DescriptorType::CombinedImageSampler,
                        count: 1000,
                    },
                    DescriptorPoolSize {
                        ty: DescriptorType::StorageImage,
                        count: 1000,
                    },
                    DescriptorPoolSize {
                        ty: DescriptorType::UniformBuffer,
                        count: 1000,
                    },
                    DescriptorPoolSize {
                        ty: DescriptorType::StorageBuffer,
                        count: 1000,
                    },
                ],
                max_sets: 1000,
                flags: DescriptorPoolFlags::FREE_DESCRIPTOR,
                label: Some("Main Descriptor Set".to_string()),
            },
        );

        //
        // Camera Set
        //

        // Dummy Camera, we'll use ECS later on
        let camera = OrthographicCamera::new(
            extent.width,
            extent.height,
        );

        let mut camera_buffers = Vec::new();

        for _ in 0..renderer.get().get_max_frames_in_flight() {
            let mut buffer = Buffer::new(
                device.clone(),
                MVBufferCreateInfo {
                    instance_size: 128,
                    instance_count: 1,
                    buffer_usage: BufferUsage::UNIFORM_BUFFER,
                    memory_properties: MemoryProperties::HOST_VISIBLE
                        | MemoryProperties::HOST_COHERENT,
                    minimum_alignment: 1,
                    memory_usage: gpu_alloc::UsageFlags::HOST_ACCESS,
                    label: Some("Camera Uniform Buffer".to_string()),
                },
            );

            let matrices = [camera.get_view(), camera.get_projection()];
            let bytes = unsafe { std::slice::from_raw_parts(matrices.as_ptr() as *const u8, 128) };

            buffer.write(bytes, 0, None);
            camera_buffers.push(buffer);
        }

        let mut camera_sets = Vec::new();

        for index in 0..renderer.get().get_max_frames_in_flight() {
            let mut camera_set = DescriptorSet::new(
                device.clone(),
                MVDescriptorSetCreateInfo {
                    pool: descriptor_pool.clone(),
                    bindings: vec![DescriptorSetLayoutBinding {
                        index: 0,
                        stages: ShaderStage::Vertex,
                        ty: DescriptorType::UniformBuffer,
                        count: 1,
                    }],
                    label: Some("Camera Set".to_string()),
                },
            );

            camera_set.add_buffer(0, &camera_buffers[index as usize], 0, 128);
            camera_set.build();

            camera_sets.push(camera_set);
        }

        //
        // Transforms
        //

        let mut transforms_buffers = Vec::new();
        for _ in 0..renderer.get().get_max_frames_in_flight() {
            transforms_buffers.push(Self::create_transform_buffer(device.clone()));
        }

        let mut transforms_sets = Vec::new();
        for index in 0..renderer.get().get_max_frames_in_flight() {
            let mut set = DescriptorSet::new(
                device.clone(),
                MVDescriptorSetCreateInfo {
                    pool: descriptor_pool.clone(),
                    bindings: vec![DescriptorSetLayoutBinding {
                        index: 0,
                        stages: ShaderStage::Vertex,
                        ty: DescriptorType::StorageBuffer,
                        count: 1,
                    }],
                    label: Some("Transforms set".to_string()),
                },
            );

            set.add_buffer(
                0,
                &transforms_buffers[index as usize],
                0,
                transforms_buffers[index as usize].get_size(),
            );
            set.build();

            transforms_sets.push(set);
        }

        let default_sampler = Sampler::new(device.clone(), MVSamplerCreateInfo {
            address_mode: SamplerAddressMode::ClampToEdge,
            filter_mode: Filter::Nearest,
            mipmap_mode: MipmapMode::Nearest,
            anisotropy: false,
            label: None,
        });

        let default_image = Image::new(device.clone(), MVImageCreateInfo {
            size: Extent2D { width: 2, height: 2 },
            format: ImageFormat::R8G8B8A8,
            usage: ImageUsage::SAMPLED | ImageUsage::STORAGE,
            memory_properties: MemoryProperties::DEVICE_LOCAL,
            aspect: ImageAspect::COLOR,
            tiling: ImageTiling::Optimal,
            layer_count: 1,
            image_type: ImageType::Image2D,
            cubemap: false,
            memory_usage_flags: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
            data: Some(vec![255u8, 0, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 255u8, 0, 255, 255]),
            label: Some("Default image".to_string()),
        });
        default_image.transition_layout(ImageLayout::ShaderReadOnlyOptimal, None, AccessFlags::empty(), AccessFlags::empty());

        let mut atlas_sets = Vec::new();
        for index in 0..renderer.get().get_max_frames_in_flight() {
            let mut set = DescriptorSet::new(
                device.clone(),
                MVDescriptorSetCreateInfo {
                    pool: descriptor_pool.clone(),
                    bindings: vec![DescriptorSetLayoutBinding {
                        index: 0,
                        stages: ShaderStage::Fragment,
                        ty: DescriptorType::CombinedImageSampler,
                        count: 1,
                    }],
                    label: Some("Atlas set".to_string()),
                },
            );

            set.add_image(
                0,
                &default_image,
                &default_sampler,
                ImageLayout::ShaderReadOnlyOptimal
            );
            set.build();

            atlas_sets.push(set);
        }


        //
        // Mesh
        //

        let vertices = vec![
            Vertex {
                position: Vec3::new(-1.0, 1.0, 0.0),
                tex_coord: Vec2::new(0.0, 0.0),
            }, // 0
            Vertex {
                position: Vec3::new(-1.0, -1.0, 0.0),
                tex_coord: Vec2::new(0.0, 1.0),
            }, // 1
            Vertex {
                position: Vec3::new(1.0, -1.0, 0.0),
                tex_coord: Vec2::new(1.0, 1.0),
            }, // 2
            Vertex {
                position: Vec3::new(1.0, 1.0, 0.0),
                tex_coord: Vec2::new(1.0, 0.0),
            }, // 3
        ];

        let vertices_bytes = unsafe {
            std::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * std::mem::size_of::<Vertex>(),
            )
        };
        let indices = vec![0u32, 1, 2, 0, 2, 3];

        let quad_mesh = Mesh::new(
            device.clone(),
            vertices_bytes,
            4,
            Some(&indices),
            Some("Main Quad Vertex Buffer".to_string()),
        );

        //
        // Framebuffer
        //
        let mut geometry_framebuffers = Vec::new();
        for _ in 0..renderer.get().get_max_frames_in_flight() {
            let framebuffer = Framebuffer::new(
                device.clone(),
                MVFramebufferCreateInfo {
                    attachment_formats: vec![ImageFormat::R16G16B16A16, ImageFormat::D16],
                    extent,
                    image_usage_flags: ImageUsage::TRANSFER_SRC,
                    render_pass_info: None,
                    label: Some("Geometry Framebuffer".to_string()),
                },
            );

            geometry_framebuffers.push(framebuffer);
        }

        //
        // Pipeline
        //

        // Shaders
        let vertex_shader = renderer.get().compile_shader(
            include_str!("shaders/default.vert"),
            ShaderKind::Vertex,
            Some("Default Quad Vertex Shader".to_string()),
            &[],
        );
        let fragment_shader = renderer.get().compile_shader(
            include_str!("shaders/default.frag"),
            ShaderKind::Fragment,
            Some("Default Quad Fragment Shader".to_string()),
            &[],
        );

        let default_pipeline = Pipeline::<Graphics>::new(
            device.clone(),
            MVGraphicsPipelineCreateInfo {
                shaders: vec![vertex_shader, fragment_shader],
                attributes: Vertex::get_attribute_description(),
                extent,
                topology: Topology::Triangle,
                cull_mode: CullMode::Back,
                enable_depth_test: true,
                depth_clamp: false,
                blending_enable: false,
                descriptor_sets: vec![camera_sets[0].get_layout(), transforms_sets[0].get_layout(), atlas_sets[0].get_layout()],
                push_constants: vec![],
                framebuffer: geometry_framebuffers[0].clone(),
                color_attachments_count: 1,
                label: Some("Default Quad Pipeline".to_string()),
            },
        );

        let manager = AssetManager::new(device.clone(), 1);

        // TODO: delete this
        let handle = manager.create_asset("texture.png", AssetType::Texture);

        handle.load();

        Self {
            extent,
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
            geometry_framebuffers,
            manager,
            handle,
            atlas_sets,
            default_sampler,
            default_image,
        }
    }

    pub fn get_atlas_sets(&mut self) -> &mut Vec<DescriptorSet>
    {
        &mut self.atlas_sets
    }

    pub fn get_sampler(&self) -> &Sampler
    {
        &self.default_sampler
    }

    pub fn draw(&mut self) {
        let current_frame = self.core_renderer.get_mut().get_current_frame_index();
        let cmd = unsafe { Unsafe::cast_static(self.core_renderer.get_mut().get_current_command_buffer()) };

        let swapchain = self.core_renderer.get_mut().get_swapchain_mut();
        let geometry_framebuffer = &self.geometry_framebuffers[current_frame as usize];

        // Push data to the storage buffer
        let bytes = unsafe {
            std::slice::from_raw_parts(
                self.transforms.as_ptr() as *const u8,
                self.transforms.len() * mem::size_of::<Transform>(),
            )
        };
        self.transforms_buffers[current_frame as usize].write(bytes, 0, None);

        // GEOMETRY PASS
        geometry_framebuffer.begin_render_pass(
            cmd,
            &[
                ClearColor::Color([0.1, 0.1, 0.1, 1.0]),
                ClearColor::Depth {
                    depth: 1.0,
                    stencil: 0,
                },
            ],
            self.extent,
        );

        self.main_pipeline.bind(cmd);

        self.camera_sets[current_frame as usize].bind(cmd, &self.main_pipeline, 0);
        self.transforms_sets[current_frame as usize].bind(cmd, &self.main_pipeline, 1);
        self.atlas_sets[current_frame as usize].bind(cmd, &self.main_pipeline, 2);

        self.quad_mesh
            .draw_instanced(cmd, 0, self.transforms.len() as u32);

        geometry_framebuffer.end_render_pass(cmd);

        // Clear all data
        self.transforms.clear();
    }

    pub fn add_quad(&mut self, mut transform: Transform) {
        if self.transforms.len() as u32 > MAX_BATCH_SIZE {
            log::error!("Todo: multiple batches");
            panic!();
        }

        self.transforms.push(transform);
    }

    fn create_transform_buffer(device: Device) -> Buffer {
        Buffer::new(
            device.clone(),
            MVBufferCreateInfo {
                instance_size: 100 * 100 * (mem::size_of::<Transform>() as u64),
                instance_count: 1,
                buffer_usage: BufferUsage::STORAGE_BUFFER,
                memory_properties: MemoryProperties::DEVICE_LOCAL,
                minimum_alignment: 1,
                memory_usage: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
                label: Some("Matrix Storage Buffer".to_string()),
            },
        )
    }

    pub fn resize(&mut self, extent: Extent2D) {
        self.extent = extent;
        self.geometry_framebuffers.clear();

        for _ in 0..self.core_renderer.get().get_max_frames_in_flight() {
            let framebuffer = Framebuffer::new(
                self.device.clone(),
                MVFramebufferCreateInfo {
                    attachment_formats: vec![ImageFormat::R16G16B16A16, ImageFormat::D16],
                    extent,
                    image_usage_flags: ImageUsage::TRANSFER_SRC,
                    render_pass_info: None,
                    label: Some("Geometry Framebuffer".to_string()),
                },
            );

            self.geometry_framebuffers.push(framebuffer);
        }

        let vertex_shader = self.core_renderer.get().compile_shader(
            include_str!("shaders/default.vert"),
            ShaderKind::Vertex,
            Some("Default Quad Vertex Shader".to_string()),
            &[],
        );
        let fragment_shader = self.core_renderer.get().compile_shader(
            include_str!("shaders/default.frag"),
            ShaderKind::Fragment,
            Some("Default Quad Fragment Shader".to_string()),
            &[],
        );

        self.main_pipeline = Pipeline::<Graphics>::new(
            self.device.clone(),
            MVGraphicsPipelineCreateInfo {
                shaders: vec![vertex_shader, fragment_shader],
                attributes: Vertex::get_attribute_description(),
                extent,
                topology: Topology::Triangle,
                cull_mode: CullMode::Back,
                enable_depth_test: true,
                depth_clamp: false,
                blending_enable: false,
                descriptor_sets: vec![self.camera_sets[0].get_layout(), self.transforms_sets[0].get_layout(), self.atlas_sets[0].get_layout()],
                push_constants: vec![],
                framebuffer: self.geometry_framebuffers[0].clone(),
                color_attachments_count: 1,
                label: Some("Default Quad Pipeline".to_string()),
            },
        );
    }
}
