use crate::font::{AtlasData, PreparedAtlasData};
use bytebuffer::ByteBuffer;
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
use mvcore::render::backend::image::{
    AccessFlags, Image, ImageAspect, ImageFormat, ImageLayout, ImageTiling, ImageType, ImageUsage,
    MVImageCreateInfo,
};
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
use mvutils::hashers::U32IdentityHasher;
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use mvutils::utils::TetrahedronOp;
use shaderc::ShaderKind;
use std::cmp::Ordering;
use std::sync::Arc;

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
        tex_id: Option<u16>,
        tex_coord: Vec4,
        color: Vec4,
        blending: f32,
    },
    RoundedRect {
        position: Vec3,
        rotation: Vec3,
        scale: Vec2,
        border_radius: f32,
        smoothness: i32,
        tex_id: Option<u16>,
        tex_coord: Vec4,
        color: Vec4,
        blending: f32,
    },
    Text {
        position: Vec3,
        rotation: Vec3,
        height: f32,
        font_id: u16,
        text: String,
        color: Vec4,
    },
}

pub enum SamplerType {
    Linear,
    Nearest,
}

#[derive(Debug)]
#[repr(C)]
struct Rectangle {
    pub position: Vec4,
    pub rotation: Vec4,
    pub tex_coord: Vec4,
    pub color: Vec4,
    pub scale: Vec2,
    pub tex_id: i32,
    pub blending: f32,
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
    pub _align: Vec2,
    pub blending: f32,
    pub tex_id: i32,
}

const INITIAL_BATCH_SIZE: u64 = 1000;
const BATCH_GROWTH_FACTOR: f64 = 1.6180339887;
const MAX_BATCH_SIZE: u64 = 1000000;

#[repr(C)]
struct CameraBuffer {
    pub view_matrix: Mat4,
    pub proj_matrix: Mat4,
    pub screen_size: Vec2,
}

pub struct GameRenderer2D {
    device: Device,
    core_renderer: Arc<DangerousCell<Renderer>>,

    rectangle_mesh: Mesh,
    rounded_rect_mesh: Mesh,

    camera_sets: Vec<DescriptorSet>,
    camera_buffers: Vec<Buffer>,
    descriptor_pool: DescriptorPool,

    rectangles: Vec<Rectangle>,
    rectangle_buffers: Vec<Buffer>,
    rectangle_sets: Vec<DescriptorSet>,

    rounded_rects: Vec<RoundedRect>,
    rounded_rect_buffers: Vec<Buffer>,
    rounded_rect_sets: Vec<DescriptorSet>,

    rectangle_pipeline: Pipeline<Graphics>,
    rounded_rect_pipeline: Pipeline<Graphics>,

    geometry_framebuffers: Vec<Framebuffer>,
    extent: Extent2D,
    nearest_sampler: Sampler,
    linear_sampler: Sampler,
    atlas_sets: Vec<DescriptorSet>,

    font_data: hashbrown::HashMap<u32, Arc<PreparedAtlasData>, U32IdentityHasher>,

    max_textures: u32,
    max_fonts: u32,
}

impl GameRenderer2D {
    pub fn get_geometry_image(&self, frame_index: usize) -> Image {
        self.geometry_framebuffers[frame_index].get_image(0).clone()
    }

    pub fn get_extent(&self) -> &Extent2D {
        &self.extent
    }

    pub fn new(
        device: Device,
        renderer: Arc<DangerousCell<Renderer>>,
        extent: Extent2D,
        max_textures: u32,
        max_fonts: u32,
    ) -> Self {
        let max_textures = if max_textures == 0 {
            65536
        } else {
            max_textures.clamp(1, 65536)
        };
        let max_fonts = if max_fonts == 0 {
            65536
        } else {
            max_fonts.clamp(1, 65536)
        };

        //
        // Pool
        //

        let descriptor_pool = DescriptorPool::new(
            device.clone(),
            MVDescriptorPoolCreateInfo {
                sizes: vec![
                    DescriptorPoolSize {
                        ty: DescriptorType::CombinedImageSampler,
                        count: max_textures + max_fonts,
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

        // TODO: Dummy Camera, we'll use ECS later on
        let camera = OrthographicCamera::new(extent.width, extent.height);

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
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    &camera_buffer as *const CameraBuffer as *const u8,
                    size_of::<CameraBuffer>(),
                )
            };

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

            camera_set.add_buffer(
                0,
                &camera_buffers[index as usize],
                0,
                size_of::<CameraBuffer>() as u64,
            );
            camera_set.build();

            camera_sets.push(camera_set);
        }

        //
        // Rectangles
        //

        let mut rectangle_buffers = Vec::new();
        let mut rectangle_sets = Vec::new();
        for _ in 0..renderer.get().get_max_frames_in_flight() {
            let (buffer, set) = Self::create_buffer_and_set(
                device.clone(),
                descriptor_pool.clone(),
                size_of::<Rectangle>() as u64,
                "Rectangle",
            );
            rectangle_buffers.push(buffer);
            rectangle_sets.push(set);
        }

        let mut rounded_rect_buffers = Vec::new();
        let mut rounded_rect_sets = Vec::new();
        for _ in 0..renderer.get().get_max_frames_in_flight() {
            let (buffer, set) = Self::create_buffer_and_set(
                device.clone(),
                descriptor_pool.clone(),
                size_of::<RoundedRect>() as u64,
                "Rounded Rect",
            );
            rounded_rect_buffers.push(buffer);
            rounded_rect_sets.push(set);
        }

        let linear_sampler = Sampler::new(
            device.clone(),
            MVSamplerCreateInfo {
                address_mode: SamplerAddressMode::ClampToEdge,
                filter_mode: Filter::Linear,
                mipmap_mode: MipmapMode::Linear,
                anisotropy: false,
                label: None,
            },
        );

        let nearest_sampler = Sampler::new(
            device.clone(),
            MVSamplerCreateInfo {
                address_mode: SamplerAddressMode::ClampToEdge,
                filter_mode: Filter::Nearest,
                mipmap_mode: MipmapMode::Nearest,
                anisotropy: false,
                label: None,
            },
        );

        let mut atlas_sets = Vec::new();
        for index in 0..renderer.get().get_max_frames_in_flight() {
            let mut set = DescriptorSet::new(
                device.clone(),
                MVDescriptorSetCreateInfo {
                    pool: descriptor_pool.clone(),
                    bindings: vec![
                        DescriptorSetLayoutBinding {
                            index: 0,
                            stages: ShaderStage::Fragment,
                            ty: DescriptorType::CombinedImageSampler,
                            count: max_textures,
                        },
                        DescriptorSetLayoutBinding {
                            index: 1,
                            stages: ShaderStage::Fragment,
                            ty: DescriptorType::CombinedImageSampler,
                            count: max_fonts,
                        },
                    ],
                    label: Some("Atlas set".to_string()),
                },
            );

            for i in 0..max_textures {
                set.add_image(
                    0,
                    renderer.get().get_missing_texture(),
                    &nearest_sampler,
                    ImageLayout::ShaderReadOnlyOptimal,
                );
            }
            for i in 0..max_fonts {
                set.add_image(
                    1,
                    renderer.get().get_missing_texture(),
                    &linear_sampler,
                    ImageLayout::ShaderReadOnlyOptimal,
                );
            }
            set.build();

            atlas_sets.push(set);
        }

        //
        // Meshes
        //

        let rectangle_vertices = vec![
            Vertex {
                position: Vec3::new(0.0, 1.0, 0.0),
            }, // 0
            Vertex {
                position: Vec3::new(0.0, 0.0, 0.0),
            }, // 1
            Vertex {
                position: Vec3::new(1.0, 0.0, 0.0),
            }, // 2
            Vertex {
                position: Vec3::new(1.0, 1.0, 0.0),
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
            Some("GameRenderer2D Rectangle Mesh".to_string()),
        );

        let rounded_rect_vertices_bytes = [0u8; 12];
        let rounded_rect_indices = vec![0u32];

        let rounded_rect_mesh = Mesh::new(
            device.clone(),
            &rounded_rect_vertices_bytes,
            1,
            Some(&rounded_rect_indices),
            Some("GameRenderer2D Rounded Rect Mesh".to_string()),
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
            Some("GameRenderer2D Shared Fragment Shader".to_string()),
            &[],
        );

        // Rectangle

        let rectangle_shader = renderer.get().compile_shader(
            include_str!("shaders/shapes/rectangle.vert"),
            ShaderKind::Vertex,
            Some("GameRenderer2D Rectangle Shader".to_string()),
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
                descriptor_sets: vec![
                    camera_sets[0].get_layout(),
                    rectangle_sets[0].get_layout(),
                    atlas_sets[0].get_layout(),
                ],
                push_constants: vec![],
                framebuffer: geometry_framebuffers[0].clone(),
                color_attachments_count: 1,
                label: Some("GameRenderer2D Rectangle Pipeline".to_string()),
            },
        );

        // Rounded Rect

        let rounded_rect_geometry_shader = renderer.get().compile_shader(
            include_str!("shaders/shapes/rounded_rect.geom"),
            ShaderKind::Geometry,
            Some("GameRenderer2D Rounded Rect Geometry Shader".to_string()),
            &[],
        );

        let rounded_rect_vertex_shader = renderer.get().compile_shader(
            include_str!("shaders/shapes/rounded_rect.vert"),
            ShaderKind::Vertex,
            Some("GameRenderer2D Rounded Rect Vertex Shader".to_string()),
            &[],
        );

        let rounded_rect_pipeline = Pipeline::<Graphics>::new(
            device.clone(),
            MVGraphicsPipelineCreateInfo {
                shaders: vec![
                    rounded_rect_vertex_shader,
                    rounded_rect_geometry_shader,
                    fragment_shader.clone(),
                ],
                attributes: Vertex::get_attribute_description(),
                extent,
                topology: Topology::Point,
                cull_mode: CullMode::None,
                enable_depth_test: true,
                depth_clamp: false,
                blending_enable: true,
                descriptor_sets: vec![
                    camera_sets[0].get_layout(),
                    rounded_rect_sets[0].get_layout(),
                    atlas_sets[0].get_layout(),
                ],
                push_constants: vec![],
                framebuffer: geometry_framebuffers[0].clone(),
                color_attachments_count: 1,
                label: Some("GameRenderer2D Rounded Rect Pipeline".to_string()),
            },
        );

        Self {
            extent,
            device,
            core_renderer: renderer,

            rectangle_mesh,
            rounded_rect_mesh,

            camera_sets,
            camera_buffers,
            descriptor_pool,

            rectangles: Vec::with_capacity(INITIAL_BATCH_SIZE as usize),
            rectangle_buffers,
            rectangle_sets,

            rounded_rects: Vec::with_capacity(INITIAL_BATCH_SIZE as usize),
            rounded_rect_buffers,
            rounded_rect_sets,

            rectangle_pipeline,
            rounded_rect_pipeline,

            geometry_framebuffers,
            atlas_sets,
            linear_sampler,
            nearest_sampler,

            max_textures: max_textures,
            max_fonts: max_textures,
            font_data: hashbrown::HashMap::with_hasher(U32IdentityHasher::default()),
        }
    }

    pub fn get_atlas_sets(&mut self) -> &mut Vec<DescriptorSet> {
        &mut self.atlas_sets
    }

    pub fn get_linear_sampler(&self) -> &Sampler {
        &self.linear_sampler
    }

    pub fn get_nearest_sampler(&self) -> &Sampler {
        &self.nearest_sampler
    }

    pub fn get_max_textures(&self) -> u32 {
        self.max_textures
    }

    pub fn get_max_fonts(&self) -> u32 {
        self.max_fonts
    }

    pub fn draw(&mut self) {
        let current_frame = self.core_renderer.get_mut().get_current_frame_index();
        let cmd = unsafe {
            Unsafe::cast_static(self.core_renderer.get_mut().get_current_command_buffer())
        };

        let swapchain = self.core_renderer.get_mut().get_swapchain_mut();
        let geometry_framebuffer = &self.geometry_framebuffers[current_frame as usize];

        // Push data to the storage buffer
        macro_rules! setup {
            ($frame:ident, $data:expr, $datatype:ty, $buffers:expr) => {
                if !$data.is_empty() {
                    $data.sort_unstable_by(|a, b| {
                        b.position
                            .z
                            .partial_cmp(&a.position.z)
                            .unwrap_or(Ordering::Equal)
                    });
                    let bytes = unsafe {
                        std::slice::from_raw_parts(
                            $data.as_ptr() as *const u8,
                            $data.len() * size_of::<$datatype>(),
                        )
                    };
                    $buffers[$frame as usize].write(bytes, 0, None);
                }
            };
        }

        // Call draw instanced on data
        macro_rules! draw {
            ($cmd:ident, $frame:ident, $data:expr, $pipeline:expr, $sets:expr, $mesh:expr) => {
                if !$data.is_empty() {
                    $pipeline.bind($cmd);

                    self.camera_sets[$frame as usize].bind($cmd, &$pipeline, 0);
                    $sets[$frame as usize].bind($cmd, &$pipeline, 1);
                    self.atlas_sets[$frame as usize].bind($cmd, &$pipeline, 2);

                    $mesh.draw_instanced($cmd, 0, $data.len() as u32);
                }
            };
        }

        setup!(
            current_frame,
            self.rectangles,
            Rectangle,
            self.rectangle_buffers
        );
        setup!(
            current_frame,
            self.rounded_rects,
            RoundedRect,
            self.rounded_rect_buffers
        );

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

        draw!(
            cmd,
            current_frame,
            self.rectangles,
            self.rectangle_pipeline,
            self.rectangle_sets,
            self.rectangle_mesh
        );
        draw!(
            cmd,
            current_frame,
            self.rounded_rects,
            self.rounded_rect_pipeline,
            self.rounded_rect_sets,
            self.rounded_rect_mesh
        );

        geometry_framebuffer.end_render_pass(cmd);

        // Clear all data
        self.rectangles.clear();
        self.rounded_rects.clear();
    }

    pub fn add_shape(&mut self, shape: Shape) {
        macro_rules! grow_batch {
            ($data:expr, $buffers:expr, $sets:expr, $datatype:ty, $ty:literal) => {
                let current_size = $data.capacity();
                let new_size = (((current_size as f64) * BATCH_GROWTH_FACTOR) as usize)
                    .min(MAX_BATCH_SIZE as usize);

                $data.reserve_exact(new_size - $data.len());

                self.device.wait_idle();

                for i in 0..$buffers.len() {
                    $buffers[i] = Buffer::new(
                        self.device.clone(),
                        MVBufferCreateInfo {
                            instance_size: (new_size * size_of::<$datatype>()) as u64,
                            instance_count: 1,
                            buffer_usage: BufferUsage::STORAGE_BUFFER,
                            memory_properties: MemoryProperties::DEVICE_LOCAL,
                            minimum_alignment: 1,
                            memory_usage: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
                            label: Some(format!("GameRenderer2D {} Buffer", $ty)),
                        },
                    );

                    $sets[i].update_buffer(0, &$buffers[i], 0, $buffers[i].get_size())
                }
            };
        }
        match shape {
            Shape::Rectangle {
                position,
                rotation,
                scale,
                tex_id,
                tex_coord,
                color,
                blending,
            } => {
                if self.rectangles.capacity() == self.rectangles.len() {
                    if self.rectangles.len() as u64 == MAX_BATCH_SIZE {
                        log::error!("Renderer2D: Maximum rectangle draw limit exceeded");
                        panic!();
                    }
                    grow_batch!(
                        self.rectangles,
                        self.rectangle_buffers,
                        self.rectangle_sets,
                        Rectangle,
                        "Rectangle"
                    );
                }

                let rot = rotation.to_radians();

                // TODO: far plane is hardcoded to 100.0, change it. We use half the far plane to allow for negative z values
                let rectangle = Rectangle {
                    position: Vec4::new(position.x, position.y, 50.0 - position.z, 0.0),
                    rotation: Vec4::new(rot.x, rot.y, rot.z, 0.0),
                    scale,
                    tex_coord,
                    color,
                    tex_id: tex_id.map(|id| id as i32).unwrap_or(-1),
                    blending: blending.clamp(0.0, 1.0),
                };

                self.rectangles.push(rectangle);
            }
            Shape::RoundedRect {
                position,
                rotation,
                scale,
                border_radius,
                smoothness,
                tex_coord,
                color,
                tex_id,
                blending,
            } => {
                if self.rounded_rects.capacity() == self.rounded_rects.len() {
                    if self.rounded_rects.len() as u64 == MAX_BATCH_SIZE {
                        log::error!("Renderer2D: Maximum rounded rectangle draw limit exceeded");
                        panic!();
                    }
                    grow_batch!(
                        self.rounded_rects,
                        self.rounded_rect_buffers,
                        self.rounded_rect_sets,
                        RoundedRect,
                        "Rounded Rect"
                    );
                }

                // TODO: far plane is hardcoded to 100.0, change it. We use half the far plane to allow for negative z values
                let rounded_rect = RoundedRect {
                    position: Vec4::new(position.x, position.y, 50.0 - position.z, 0.0),
                    rotation: rotation.to_radians().into(),
                    scale,
                    border_radius,
                    smoothness: smoothness.clamp(1, 20),
                    tex_coord,
                    color,
                    tex_id: tex_id.map(|id| id as i32).unwrap_or(-1),
                    blending,
                    _align: Default::default(),
                };

                self.rounded_rects.push(rounded_rect);
            }
            Shape::Text {
                position,
                rotation,
                height,
                font_id,
                text,
                color,
            } => {
                while self.rectangles.capacity() <= self.rectangles.len() + text.len() {
                    if self.rectangles.len() as u64 == MAX_BATCH_SIZE {
                        log::error!("Renderer2D: Maximum rectangle draw limit exceeded");
                        panic!();
                    }
                    grow_batch!(
                        self.rectangles,
                        self.rectangle_buffers,
                        self.rectangle_sets,
                        Rectangle,
                        "Rectangle"
                    );
                }

                if let Some(atlas) = self.font_data.get(&(font_id as u32)).cloned() {
                    let mut font_scale = 1.0 / (atlas.metrics.ascender - atlas.metrics.descender);
                    font_scale *= height as f64 / atlas.metrics.line_height;
                    let space_advance = atlas.find_glyph(' ').unwrap().advance; // TODO

                    let mut x = 0.0;
                    let mut y = 0.0;

                    for char in text.chars() {
                        if (char == '\t') {
                            x += 6.0 + space_advance * font_scale;
                            continue;
                        } else if (char == ' ') {
                            x += space_advance * font_scale;
                            continue;
                        } else if (char == '\n') {
                            x = 0.0;
                            y -= font_scale * atlas.metrics.line_height;
                            continue;
                        }

                        let glyph = if let Some(glyph) = atlas.find_glyph(char) {
                            glyph
                        } else {
                            atlas.find_glyph('?').unwrap_or_else(|| {
                                log::error!("Font atlas missing 'missing character' glyph");
                                panic!()
                            })
                        };

                        let bounds_plane = &glyph.plane_bounds;
                        let bounds_atlas = &glyph.atlas_bounds;

                        let mut tex_coords = Vec4::new(
                            bounds_atlas.left as f32,
                            (atlas.atlas.height as f64 - bounds_atlas.top) as f32,
                            (bounds_atlas.right - bounds_atlas.left) as f32,
                            (bounds_atlas.top - bounds_atlas.bottom) as f32,
                        );

                        tex_coords.x /= atlas.atlas.width as f32;
                        tex_coords.y /= atlas.atlas.height as f32;
                        tex_coords.z /= atlas.atlas.width as f32;
                        tex_coords.w /= atlas.atlas.height as f32;

                        let mut scale = Vec2::new(
                            (bounds_plane.right - bounds_plane.left) as f32,
                            (bounds_plane.top - bounds_plane.bottom) as f32,
                        );
                        scale.x = scale.x * font_scale as f32;
                        scale.y = scale.y * font_scale as f32;

                        let rot = rotation.to_radians();

                        let y_offset: f32 = (1.0 - bounds_plane.bottom) as f32 * scale.y;

                        // TODO: far plane is hardcoded to 100.0, change it. We use half the far plane to allow for negative z values
                        let rectangle = Rectangle {
                            position: Vec4::new(
                                x as f32 + position.x,
                                y as f32 + position.y - y_offset,
                                50.0 - position.z,
                                0.0,
                            ),
                            rotation: Vec4::new(rot.x, rot.y, rot.z, 0.0),
                            scale,
                            tex_coord: tex_coords,
                            color,
                            tex_id: font_id as i32,
                            blending: -1.0,
                        };

                        self.rectangles.push(rectangle);

                        x += glyph.advance * font_scale;
                    }
                }
            }
        }
    }

    pub fn remove_texture(&mut self, index: u32) {
        self.device.wait_idle();
        for atlas in &mut self.atlas_sets {
            atlas.update_image_array(
                0,
                index,
                self.core_renderer.get().get_missing_texture(),
                &self.nearest_sampler,
                ImageLayout::ShaderReadOnlyOptimal,
            );
        }
    }

    pub fn set_texture(&mut self, index: u32, texture: &Image, sampler_type: SamplerType) {
        self.device.wait_idle();
        for atlas in &mut self.atlas_sets {
            atlas.update_image_array(
                0,
                index,
                texture,
                match sampler_type {
                    SamplerType::Linear => &self.linear_sampler,
                    SamplerType::Nearest => &self.nearest_sampler,
                },
                ImageLayout::ShaderReadOnlyOptimal,
            );
        }
    }

    pub fn set_font(&mut self, index: u32, texture: &Image, data: Arc<PreparedAtlasData>) {
        self.device.wait_idle();
        for atlas in &mut self.atlas_sets {
            atlas.update_image_array(
                1,
                index,
                texture,
                &self.linear_sampler,
                ImageLayout::ShaderReadOnlyOptimal,
            );
        }
        self.font_data.insert(index, data);
    }

    fn create_buffer_and_set(
        device: Device,
        descriptor_pool: DescriptorPool,
        struct_size: u64,
        ty: &'static str,
    ) -> (Buffer, DescriptorSet) {
        let buffer = Buffer::new(
            device.clone(),
            MVBufferCreateInfo {
                instance_size: INITIAL_BATCH_SIZE * struct_size,
                instance_count: 1,
                buffer_usage: BufferUsage::STORAGE_BUFFER,
                memory_properties: MemoryProperties::DEVICE_LOCAL,
                minimum_alignment: 1,
                memory_usage: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
                label: Some(format!("GameRenderer2D {ty} Buffer")),
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
                label: Some(format!("GameRenderer2D {ty} Set")),
            },
        );

        set.add_buffer(0, &buffer, 0, buffer.get_size());
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
                descriptor_sets: vec![
                    self.camera_sets[0].get_layout(),
                    self.rectangle_sets[0].get_layout(),
                    self.atlas_sets[0].get_layout(),
                ],
                push_constants: vec![],
                framebuffer: self.geometry_framebuffers[0].clone(),
                color_attachments_count: 1,
                label: Some("Default Quad Pipeline".to_string()),
            },
        );
    }
}
