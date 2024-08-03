use std::sync::Arc;

use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use mvutils::utils::TetrahedronOp;
use shaderc::ShaderKind;
use mvcore::asset::asset::AssetType;
use mvcore::asset::manager::{AssetHandle, AssetManager};
use mvcore::math::mat::Mat4;
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
}

impl Vertex {
    pub fn get_attribute_description() -> Vec<AttributeType> {
        vec![AttributeType::Float32x3]
    }
}

pub enum Shape {
    Rectangle {
        position: Vec3,
        rotation: Vec3,
        scale: Vec2,
        tex_coord: Vec4,
        color: Vec4,
    },
    Triangle {
        vertices: [Vec2; 3],
        translation: Vec3,
        scale: Vec2,
        rotation: Vec3,
        tex_coord: Vec4,
        color: Vec4,
    },
    RoundedRect {
        position: Vec3,
        rotation: Vec3,
        scale: Vec2,
        border_radius: f32,
        smoothness: i32,
        tex_coord: Vec4,
        color: Vec4,
    }
}

#[repr(C)]
struct Rectangle {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec2,
    pub tex_coord: Vec4,
    pub color: Vec4,
}

#[repr(C)]
struct Triangle {
    pub vertices: [Vec4; 3],
    pub translation: Vec3,
    pub scale: Vec2,
    pub rotation: Vec3,
    pub color: Vec4,
}

#[repr(C)]
struct RoundedRect {
    pub tex_coord: Vec4,
    pub color: Vec4,
    pub position: Vec4,
    pub rotation: Vec4,
    pub scale: Vec2,
    pub border_radius: f32,
    pub smoothness: i32,
}

static MAX_BATCH_SIZE: u64 = 10000;

#[repr(C)]
struct CameraBuffer
{
    pub view_matrix: Mat4,
    pub proj_matrix: Mat4,
    pub screen_size: Vec2
}

pub struct Renderer2D {
    device: Device,
    core_renderer: Arc<DangerousCell<Renderer>>,

    rectangle_mesh: Mesh,
    triangle_mesh: Mesh,
    rounded_rect_mesh: Mesh,

    camera_sets: Vec<DescriptorSet>,
    camera_buffers: Vec<Buffer>,
    descriptor_pool: DescriptorPool,

    // TODO: make transparency work properly, potentially remove triangles and assume that only rects can be drawn behind other objects
    rectangles: Vec<Rectangle>,
    transparent_rectangles: Vec<Rectangle>,
    rectangle_buffers: Vec<Buffer>,
    rectangle_sets: Vec<DescriptorSet>,

    triangles: Vec<Triangle>,
    transparent_triangles: Vec<Triangle>,
    triangle_buffers: Vec<Buffer>,
    triangle_sets: Vec<DescriptorSet>,

    rounded_rects: Vec<RoundedRect>,
    transparent_rounded_rects: Vec<RoundedRect>,
    rounded_rect_buffers: Vec<Buffer>,
    rounded_rect_sets: Vec<DescriptorSet>,

    rectangle_pipeline: Pipeline<Graphics>,
    triangle_pipeline: Pipeline<Graphics>,
    rounded_rect_pipeline: Pipeline<Graphics>,

    geometry_framebuffers: Vec<Framebuffer>,
    extent: Extent2D,
    default_sampler: Sampler,
    atlas_sets: Vec<DescriptorSet>,
}

impl Renderer2D {

    pub fn get_geometry_image(&self, frame_index: usize) -> Image {
        self.geometry_framebuffers[frame_index].get_image(0).clone()
    }

    pub fn new(device: Device, renderer: Arc<DangerousCell<Renderer>>, extent: Extent2D, preallocate: bool) -> Self {

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
                    instance_size: size_of::<CameraBuffer>() as u64,
                    instance_count: 1,
                    buffer_usage: BufferUsage::UNIFORM_BUFFER,
                    memory_properties: MemoryProperties::HOST_VISIBLE
                        | MemoryProperties::HOST_COHERENT,
                    minimum_alignment: 1,
                    memory_usage: gpu_alloc::UsageFlags::HOST_ACCESS,
                    label: Some("Camera Uniform Buffer".to_string()),
                },
            );

            let camera_buffer: CameraBuffer = CameraBuffer {
                view_matrix: camera.get_view(),
                proj_matrix: camera.get_projection(),
                screen_size: Vec2::new(600.0, 600.0),
            };
            let bytes = unsafe { std::slice::from_raw_parts(&camera_buffer as *const CameraBuffer as *const u8, size_of::<CameraBuffer>()) };

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

            camera_set.add_buffer(0, &camera_buffers[index as usize], 0, size_of::<CameraBuffer>() as u64);
            camera_set.build();

            camera_sets.push(camera_set);
        }

        //
        // Rectangles
        //

        let mut rectangle_buffers = Vec::new();
        let mut rectangle_sets = Vec::new();
        for _ in 0..renderer.get().get_max_frames_in_flight() {
            let (buffer, set) = Self::create_buffer_and_set(device.clone(), descriptor_pool.clone(), size_of::<Rectangle>() as u64, "Rectangle");
            rectangle_buffers.push(buffer);
            rectangle_sets.push(set);
        }

        let mut triangle_buffers = Vec::new();
        let mut triangle_sets = Vec::new();
        for _ in 0..renderer.get().get_max_frames_in_flight() {
            let (buffer, set) = Self::create_buffer_and_set(device.clone(), descriptor_pool.clone(), size_of::<Triangle>() as u64, "Triangle");
            triangle_buffers.push(buffer);
            triangle_sets.push(set);
        }

        let mut rounded_rect_buffers = Vec::new();
        let mut rounded_rect_sets = Vec::new();
        for _ in 0..renderer.get().get_max_frames_in_flight() {
            let (buffer, set) = Self::create_buffer_and_set(device.clone(), descriptor_pool.clone(), size_of::<RoundedRect>() as u64, "Rounded Rect");
            rounded_rect_buffers.push(buffer);
            rounded_rect_sets.push(set);
        }

        let default_sampler = Sampler::new(device.clone(), MVSamplerCreateInfo {
            address_mode: SamplerAddressMode::ClampToEdge,
            filter_mode: Filter::Nearest,
            mipmap_mode: MipmapMode::Nearest,
            anisotropy: false,
            label: None,
        });

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
                &renderer.get().get_missing_texture(),
                &default_sampler,
                ImageLayout::ShaderReadOnlyOptimal
            );
            set.build();

            atlas_sets.push(set);
        }


        //
        // Meshes
        //

        let rectangle_vertices = vec![
            Vertex {
                position: Vec3::new(-1.0, 1.0, 0.0)
            }, // 0
            Vertex {
                position: Vec3::new(-1.0, -1.0, 0.0)
            }, // 1
            Vertex {
                position: Vec3::new(1.0, -1.0, 0.0)
            }, // 2
            Vertex {
                position: Vec3::new(1.0, 1.0, 0.0)
            }, // 3
        ];

        let rectangle_vertices_bytes = unsafe {
            std::slice::from_raw_parts(
                rectangle_vertices.as_ptr() as *const u8,
                rectangle_vertices.len() * size_of::<Vertex>(),
            )
        };
        let rectangle_indices = vec![0u32, 1, 2, 0, 2, 3];

        let rectangle_mesh = Mesh::new(
            device.clone(),
            rectangle_vertices_bytes,
            4,
            Some(&rectangle_indices),
            Some("Render2D Rectangle Mesh".to_string()),
        );

        let triangle_vertices_bytes = [0u8; 36];
        let triangle_indices = vec![0u32, 1, 2];

        let triangle_mesh = Mesh::new(
            device.clone(),
            &triangle_vertices_bytes,
            3,
            Some(&triangle_indices),
            Some("Render2D Triangle Mesh".to_string()),
        );

        let rounded_rect_vertices_bytes = [0u8; 12];
        let rounded_rect_indices = vec![0u32];

        let rounded_rect_mesh = Mesh::new(
            device.clone(),
            &rounded_rect_vertices_bytes,
            1,
            Some(&rounded_rect_indices),
            Some("Render2D Rounded Rect Mesh".to_string()),
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
        // Pipelines
        //

        // Shared fragment shader

        let fragment_shader = renderer.get().compile_shader(
            include_str!("shaders/default.frag"),
            ShaderKind::Fragment,
            Some("Render2D Shared Fragment Shader".to_string()),
            &[],
        );

        // Rectangle

        let rectangle_shader = renderer.get().compile_shader(
            include_str!("shaders/shapes/rectangle.vert"),
            ShaderKind::Vertex,
            Some("Render2D Rectangle Shader".to_string()),
            &[],
        );

        let rectangle_pipeline = Pipeline::<Graphics>::new(
            device.clone(),
            MVGraphicsPipelineCreateInfo {
                shaders: vec![rectangle_shader, fragment_shader.clone()],
                attributes: Vertex::get_attribute_description(),
                extent,
                topology: Topology::Triangle,
                cull_mode: CullMode::None,
                enable_depth_test: true,
                depth_clamp: false,
                blending_enable: true,
                descriptor_sets: vec![camera_sets[0].get_layout(), rectangle_sets[0].get_layout(), atlas_sets[0].get_layout()],
                push_constants: vec![],
                framebuffer: geometry_framebuffers[0].clone(),
                color_attachments_count: 1,
                label: Some("Render2D Rectangle Pipeline".to_string()),
            },
        );

        // Triangle

        let triangle_vertex_shader = renderer.get().compile_shader(
            include_str!("shaders/shapes/triangle.vert"),
            ShaderKind::Vertex,
            Some("Render2D Triangle Vertex Shader".to_string()),
            &[],
        );

        let triangle_pipeline = Pipeline::<Graphics>::new(
            device.clone(),
            MVGraphicsPipelineCreateInfo {
                shaders: vec![triangle_vertex_shader, fragment_shader.clone()],
                attributes: Vertex::get_attribute_description(),
                extent,
                topology: Topology::Triangle,
                cull_mode: CullMode::None,
                enable_depth_test: true,
                depth_clamp: false,
                blending_enable: true,
                descriptor_sets: vec![camera_sets[0].get_layout(), triangle_sets[0].get_layout(), atlas_sets[0].get_layout()],
                push_constants: vec![],
                framebuffer: geometry_framebuffers[0].clone(),
                color_attachments_count: 1,
                label: Some("Render2D Triangle Pipeline".to_string()),
            },
        );

        // Rounded Rect

        let rounded_rect_geometry_shader = renderer.get().compile_shader(
            include_str!("shaders/shapes/rounded_rect.geom"),
            ShaderKind::Geometry,
            Some("Render2D Rounded Rect Geometry Shader".to_string()),
            &[],
        );

        let rounded_rect_vertex_shader = renderer.get().compile_shader(
            include_str!("shaders/shapes/rounded_rect.vert"),
            ShaderKind::Vertex,
            Some("Render2D Rounded Rect Vertex Shader".to_string()),
            &[],
        );

        let rounded_rect_pipeline = Pipeline::<Graphics>::new(
            device.clone(),
            MVGraphicsPipelineCreateInfo {
                shaders: vec![rounded_rect_vertex_shader, rounded_rect_geometry_shader, fragment_shader.clone()],
                attributes: Vertex::get_attribute_description(),
                extent,
                topology: Topology::Point,
                cull_mode: CullMode::None,
                enable_depth_test: true,
                depth_clamp: false,
                blending_enable: true,
                descriptor_sets: vec![camera_sets[0].get_layout(), rounded_rect_sets[0].get_layout(), atlas_sets[0].get_layout()],
                push_constants: vec![],
                framebuffer: geometry_framebuffers[0].clone(),
                color_attachments_count: 1,
                label: Some("Render2D Rounded Rect Pipeline".to_string()),
            },
        );

        Self {
            extent,
            device,
            core_renderer: renderer,

            rectangle_mesh,
            triangle_mesh,
            rounded_rect_mesh,

            camera_sets,
            camera_buffers,
            descriptor_pool,

            rectangles: Vec::with_capacity(preallocate.yn(MAX_BATCH_SIZE as usize, 0)),
            transparent_rectangles: Vec::with_capacity(preallocate.yn(MAX_BATCH_SIZE as usize, 0)),
            rectangle_buffers,
            rectangle_sets,

            triangles: Vec::with_capacity(preallocate.yn(MAX_BATCH_SIZE as usize, 0)),
            transparent_triangles: Vec::with_capacity(preallocate.yn(MAX_BATCH_SIZE as usize, 0)),
            triangle_buffers,
            triangle_sets,

            rounded_rects: Vec::with_capacity(preallocate.yn(MAX_BATCH_SIZE as usize, 0)),
            transparent_rounded_rects: Vec::with_capacity(preallocate.yn(MAX_BATCH_SIZE as usize, 0)),
            rounded_rect_buffers,
            rounded_rect_sets,

            rectangle_pipeline,
            triangle_pipeline,
            rounded_rect_pipeline,

            geometry_framebuffers,
            atlas_sets,
            default_sampler,
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
        macro_rules! setup {
            ($frame:ident, $data:expr, $transparent_data:expr, $datatype:ty, $buffers:expr) => {
                if !$data.is_empty() {
                    let bytes = unsafe {
                        std::slice::from_raw_parts(
                            $data.as_ptr() as *const u8,
                            $data.len() * size_of::<$datatype>(),
                        )
                    };
                    $buffers[$frame as usize].write(bytes, 0, None);
                }
                if !$transparent_data.is_empty() {
                    let bytes = unsafe {
                        std::slice::from_raw_parts(
                            $transparent_data.as_ptr() as *const u8,
                            $transparent_data.len() * size_of::<$datatype>(),
                        )
                    };
                    $buffers[$frame as usize].write(bytes, MAX_BATCH_SIZE * size_of::<$datatype>() as u64, None);
                }
            };
        }

        // Call draw instanced on data
        macro_rules! draw {
            ($cmd:ident, $frame:ident, $data:expr, $offset:expr, $pipeline:expr, $sets:expr, $mesh:expr) => {
                if !$data.is_empty() {
                    $pipeline.bind($cmd);

                    self.camera_sets[$frame as usize].bind($cmd, &$pipeline, 0);
                    $sets[$frame as usize].bind($cmd, &$pipeline, 1);
                    self.atlas_sets[$frame as usize].bind($cmd, &$pipeline, 2);

                    $mesh.draw_instanced($cmd, $offset as u32, $data.len() as u32);
                }
            };
        }

        setup!(current_frame, self.rectangles, self.transparent_rectangles, Rectangle, self.rectangle_buffers);
        setup!(current_frame, self.triangles, self.transparent_triangles, Triangle, self.triangle_buffers);
        setup!(current_frame, self.rounded_rects, self.transparent_rounded_rects, RoundedRect, self.rounded_rect_buffers);

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

        draw!(cmd, current_frame, self.triangles, 0, self.triangle_pipeline, self.triangle_sets, self.triangle_mesh);
        draw!(cmd, current_frame, self.rounded_rects, 0, self.rounded_rect_pipeline, self.rounded_rect_sets, self.rounded_rect_mesh);
        draw!(cmd, current_frame, self.rectangles, 0, self.rectangle_pipeline, self.rectangle_sets, self.rectangle_mesh);

        draw!(cmd, current_frame, self.transparent_triangles, MAX_BATCH_SIZE, self.triangle_pipeline, self.triangle_sets, self.triangle_mesh);
        draw!(cmd, current_frame, self.transparent_rounded_rects, MAX_BATCH_SIZE, self.rounded_rect_pipeline, self.rounded_rect_sets, self.rounded_rect_mesh);
        draw!(cmd, current_frame, self.transparent_rectangles, MAX_BATCH_SIZE, self.rectangle_pipeline, self.rectangle_sets, self.rectangle_mesh);

        geometry_framebuffer.end_render_pass(cmd);

        // Clear all data
        self.rectangles.clear();
        self.triangles.clear();
        self.rounded_rects.clear();
        self.transparent_rectangles.clear();
        self.transparent_triangles.clear();
        self.transparent_rounded_rects.clear();
    }

    pub fn add_shape(&mut self, shape: Shape) {
        match shape {
            Shape::Rectangle { position, rotation, scale, tex_coord, color } => {
                if self.rectangles.len() as u64 > MAX_BATCH_SIZE {
                    // TODO: batching
                    log::error!("Todo: multiple batches");
                    panic!();
                }

                let rectangle = Rectangle {
                    position,
                    rotation: rotation.to_radians(),
                    scale,
                    tex_coord,
                    color,
                };
                if (color.w < 1.0) {
                    self.transparent_rectangles.push(rectangle);
                }
                else {
                    self.rectangles.push(rectangle);
                }
            }
            Shape::Triangle { vertices, translation, rotation, scale, tex_coord, color } => {
                if self.triangles.len() as u64 > MAX_BATCH_SIZE {
                    // TODO: batching
                    log::error!("Todo: multiple batches");
                    panic!();
                }

                let mut max_x = -1.0f32;
                let mut max_y = -1.0f32;
                let mut min_x = 1.0f32;
                let mut min_y = 1.0f32;

                for coord in vertices {
                    if coord.x > max_x {
                        max_x = coord.x;
                    }
                    if coord.x < min_x {
                        min_x = coord.x
                    }
                    if coord.y > max_y {
                        max_y = coord.y;
                    }
                    if coord.y < min_y {
                        min_y = coord.y
                    }
                }

                let mut vertices4 = [Vec4::default(); 3];
                for i in 0..3
                {
                    vertices4[i].x = vertices[i].x;
                    vertices4[i].y = vertices[i].y;

                    vertices4[i].z = ((vertices[i].x - min_x)/ (max_x - min_x)) * tex_coord.z + tex_coord.x;
                    vertices4[i].w = ((vertices[i].y - min_y) / (max_y - min_y)) * tex_coord.w + tex_coord.y;
                }


                let triangle = Triangle {
                    vertices: vertices4,
                    translation,
                    rotation: rotation.to_radians(),
                    scale,
                    color,
                };
                if (color.w < 1.0) {
                    self.transparent_triangles.push(triangle);
                }
                else {
                    self.triangles.push(triangle);
                }
            }
            Shape::RoundedRect { position, rotation, scale, border_radius, smoothness, tex_coord, color } => {
                if self.rounded_rects.len() as u64 > MAX_BATCH_SIZE {
                    // TODO: batching
                    log::error!("Todo: multiple batches");
                    panic!();
                }

                let rounded_rect = RoundedRect {
                    position: position.into(),
                    rotation: rotation.to_radians().into(),
                    scale,
                    border_radius,
                    smoothness: smoothness.clamp(1, 20),
                    tex_coord,
                    color,
                };
                if (color.w < 1.0) {
                    self.transparent_rounded_rects.push(rounded_rect);
                }
                else {
                    self.rounded_rects.push(rounded_rect);
                }
            }
        }
    }

    pub fn disable_texture(&mut self) {
        for atlas in &mut self.atlas_sets {
            atlas.update_image(
                0,
                self.core_renderer.get().get_empty_texture(),
                &self.default_sampler,
                ImageLayout::ShaderReadOnlyOptimal,
            );
        }
    }

    pub fn reset_texture(&mut self) {
        for atlas in &mut self.atlas_sets {
            atlas.update_image(
                0,
                self.core_renderer.get().get_missing_texture(),
                &self.default_sampler,
                ImageLayout::ShaderReadOnlyOptimal,
            );
        }
    }

    fn create_buffer_and_set(device: Device, descriptor_pool: DescriptorPool, struct_size: u64, ty: &'static str) -> (Buffer, DescriptorSet) {
        let buffer = Buffer::new(
            device.clone(),
            MVBufferCreateInfo {
                instance_size: MAX_BATCH_SIZE * 2 * struct_size, // * 2 to account for transparent object taking up to half of the buffer
                instance_count: 1,
                buffer_usage: BufferUsage::STORAGE_BUFFER,
                memory_properties: MemoryProperties::DEVICE_LOCAL,
                minimum_alignment: 1,
                memory_usage: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
                label: Some(format!("Render2D {ty} Buffer")),
            },
        );

        let mut set = DescriptorSet::new(
            device.clone(),
            MVDescriptorSetCreateInfo {
                pool: descriptor_pool,
                bindings: vec![DescriptorSetLayoutBinding {
                    index: 0,
                    stages: ShaderStage::Vertex,
                    ty: DescriptorType::StorageBuffer,
                    count: 1,
                }],
                label: Some(format!("Render2D {ty} Set")),
            },
        );

        set.add_buffer(
            0,
            &buffer,
            0,
            buffer.get_size(),
        );
        set.build();

        (buffer, set)
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
            include_str!("shaders/shapes/rectangle.vert"),
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

        self.rectangle_pipeline = Pipeline::<Graphics>::new(
            self.device.clone(),
            MVGraphicsPipelineCreateInfo {
                shaders: vec![vertex_shader, fragment_shader],
                attributes: Vertex::get_attribute_description(),
                extent,
                topology: Topology::Triangle,
                cull_mode: CullMode::Back,
                enable_depth_test: true,
                depth_clamp: false,
                blending_enable: true,
                descriptor_sets: vec![self.camera_sets[0].get_layout(), self.rectangle_sets[0].get_layout(), self.atlas_sets[0].get_layout()],
                push_constants: vec![],
                framebuffer: self.geometry_framebuffers[0].clone(),
                color_attachments_count: 1,
                label: Some("Default Quad Pipeline".to_string()),
            },
        );
    }
}
