use std::cmp::Ordering;
use crate::gpu::{CameraBuffer, Transform};
use mvcore::math::vec::{Vec2, Vec4};
use mvcore::render::backend::buffer::{Buffer, BufferUsage, MVBufferCreateInfo, MemoryProperties};
use mvcore::render::backend::descriptor_set::{
    DescriptorPool, DescriptorPoolFlags, DescriptorPoolSize, DescriptorSet,
    DescriptorSetLayoutBinding, DescriptorType, MVDescriptorPoolCreateInfo,
    MVDescriptorSetCreateInfo,
};
use mvcore::render::backend::device::Device;
use mvcore::render::backend::framebuffer::{ClearColor, Framebuffer, MVFramebufferCreateInfo};
use mvcore::render::backend::image::{Image, ImageFormat, ImageLayout, ImageUsage};
use mvcore::render::backend::pipeline::{
    AttributeType, CullMode, Graphics, MVGraphicsPipelineCreateInfo, Pipeline, Topology,
};
use mvcore::render::backend::sampler::{
    Filter, MVSamplerCreateInfo, MipmapMode, Sampler, SamplerAddressMode,
};
use mvcore::render::backend::shader::ShaderStage;
use mvcore::render::backend::Extent2D;
use mvcore::render::camera::OrthographicCamera;
use mvcore::render::renderer::Renderer;
use mvutils::unsafe_utils::{DangerousCell, Unsafe, UnsafeInto};
use shaderc::ShaderKind;
use std::sync::Arc;
use mvutils::hashers::U32IdentityHasher;
use num_traits::ToPrimitive;
use mvcore::render::font::PreparedAtlasData;
use mvcore::render::mesh::Mesh;

const INITIAL_BATCH_SIZE: u64 = 1000;
const BATCH_GROWTH_FACTOR: f64 = 1.6180339887;
const MAX_BATCH_SIZE: u64 = 1000000;

pub enum SamplerType {
    Linear,
    Nearest,
}

pub struct InputTriangle {
    pub points: [(i32, i32); 3],
    pub z: f32,
    pub transform: Transform,
    pub canvas_transform: Transform,
    pub tex_id: Option<u16>,
    pub tex_coords: Option<[(f32, f32); 3]>,
    pub blending: f32,
    pub colors: [Vec4; 3],
    pub is_font: bool,
}

impl From<InputTriangle> for Triangle {
    fn from(value: InputTriangle) -> Self {
        Triangle {
            points: value.points.map(|(x, y)| (x as f32, y as f32)),
            z: value.z,
            transform: value.transform,
            canvas_transform: value.canvas_transform,
            tex_id: value.tex_id.map(|i| i as f32).unwrap_or(-1.0),
            tex_coords: value.tex_coords.unwrap_or_default(),
            colors: value.colors,
            blending: if value.is_font { -1.0 } else { value.blending.max(0.0) },
        }
    }
}

#[repr(C)]
pub struct Triangle {
    transform: Transform,
    canvas_transform: Transform,
    colors: [Vec4; 3],
    points: [(f32, f32); 3],
    tex_coords: [(f32, f32); 3],
    z: f32,
    tex_id: f32,
    blending: f32,
}

pub struct Renderer2D {
    device: Device,
    core_renderer: Arc<DangerousCell<Renderer>>,

    mesh: Mesh,

    camera_buffers: Vec<Buffer>,
    camera_sets: Vec<DescriptorSet>,
    descriptor_pool: DescriptorPool,

    triangles: Vec<Triangle>,
    buffers: Vec<Buffer>,
    sets: Vec<DescriptorSet>,

    pipeline: Pipeline<Graphics>,

    geometry_framebuffers: Vec<Framebuffer>,
    extent: Extent2D,
    nearest_sampler: Sampler,
    linear_sampler: Sampler,
    atlas_sets: Vec<DescriptorSet>,

    font_data: hashbrown::HashMap<u32, Arc<PreparedAtlasData>, U32IdentityHasher>,

    max_textures: u32,
    max_fonts: u32,
}

impl Renderer2D {
    pub fn new(device: Device, core_renderer: Arc<DangerousCell<Renderer>>, max_textures: u32, max_fonts: u32) -> Self {
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

        let extent = core_renderer.get().get_swapchain().get_extent();

        let camera = OrthographicCamera::new(extent.width, extent.height);

        let mut camera_buffers = Vec::new();

        let max_frames = core_renderer.get().get_max_frames_in_flight();
        for _ in 0..max_frames {
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
                    label: None,
                },
            );

            let buf = CameraBuffer {
                view_matrix: camera.get_view(),
                proj_matrix: camera.get_projection(),
                screen_size: Vec2::new(extent.width as f32, extent.height as f32),
            };

            let bytes = unsafe {
                std::slice::from_raw_parts(
                    &buf as *const CameraBuffer as *const u8,
                    size_of::<CameraBuffer>(),
                )
            };

            buffer.write(bytes, 0, None);
            camera_buffers.push(buffer);
        }

        let mut camera_sets = Vec::new();

        for index in 0..max_frames {
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

        let mut triangle_buffers = Vec::new();
        let mut triangle_sets = Vec::new();
        for _ in 0..core_renderer.get().get_max_frames_in_flight() {
            let (buffer, set) = Self::create_buffer_and_set(
                device.clone(),
                descriptor_pool.clone(),
                size_of::<Triangle>() as u64,
                "Triangle",
            );
            triangle_buffers.push(buffer);
            triangle_sets.push(set);
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
        for index in 0..core_renderer.get().get_max_frames_in_flight() {
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
                    core_renderer.get().get_missing_texture(),
                    &nearest_sampler,
                    ImageLayout::ShaderReadOnlyOptimal,
                );
            }
            for i in 0..max_fonts {
                set.add_image(
                    1,
                    core_renderer.get().get_missing_texture(),
                    &linear_sampler,
                    ImageLayout::ShaderReadOnlyOptimal,
                );
            }
            set.build();

            atlas_sets.push(set);
        }

        let indices = vec![0u32, 1, 2];

        let triangle_mesh = Mesh::new(
            device.clone(),
            &[],
            3,
            Some(&indices),
            Some("Renderer2D Rectangle Mesh".to_string()),
        );

        let mut geometry_framebuffers = Vec::new();
        for _ in 0..max_frames {
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

        let vertex_shader = core_renderer.get().compile_shader(
            include_str!("shaders/default.vert"),
            ShaderKind::Vertex,
            Some("Renderer2D Shared Vertex Shader".to_string()),
            &[],
        );

        let fragment_shader = core_renderer.get().compile_shader(
            include_str!("shaders/default.frag"),
            ShaderKind::Fragment,
            Some("Renderer2D Shared Fragment Shader".to_string()),
            &[],
        );


        let pipeline = Pipeline::<Graphics>::new(
            device.clone(),
            MVGraphicsPipelineCreateInfo {
                shaders: vec![vertex_shader, fragment_shader],
                attributes: vec![],
                extent,
                topology: Topology::Triangle,
                cull_mode: CullMode::None,
                enable_depth_test: true,
                depth_clamp: false,
                blending_enable: true,
                descriptor_sets: vec![
                    camera_sets[0].get_layout(),
                    triangle_sets[0].get_layout(),
                    atlas_sets[0].get_layout(),
                ],
                push_constants: vec![],
                framebuffer: geometry_framebuffers[0].clone(),
                color_attachments_count: 1,
                label: Some("Renderer2D Triangle Pipeline".to_string()),
            },
        );

        Self {
            device,
            core_renderer,
            mesh: triangle_mesh,
            camera_buffers,
            camera_sets,
            descriptor_pool,
            triangles: Vec::with_capacity(INITIAL_BATCH_SIZE as usize),
            buffers: triangle_buffers,
            sets: triangle_sets,
            pipeline,
            geometry_framebuffers,
            extent,
            nearest_sampler,
            linear_sampler,
            atlas_sets,
            font_data: hashbrown::HashMap::with_hasher(U32IdentityHasher::default()),
            max_textures,
            max_fonts,
        }
    }

    pub fn get_geometry_image(&self, frame_index: usize) -> Image {
        self.geometry_framebuffers[frame_index].get_image(0).clone()
    }

    pub fn get_extent(&self) -> &Extent2D {
        &self.extent
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

        if !self.triangles.is_empty() {
            self.triangles.sort_unstable_by(|a, b| {
                b.z
                    .partial_cmp(&a.z)
                    .unwrap_or(Ordering::Equal)
            });
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    self.triangles.as_ptr() as *const u8,
                    self.triangles.len() * size_of::<Triangle>(),
                )
            };
            self.buffers[current_frame as usize].write(bytes, 0, None);
        }

        // GEOMETRY PASS
        geometry_framebuffer.begin_render_pass(
            cmd,
            &[
                ClearColor::Color([0.0, 0.0, 0.0, 1.0]),
                ClearColor::Depth {
                    depth: 1.0,
                    stencil: 0,
                },
            ],
            self.extent,
        );

        if !self.triangles.is_empty() {
            self.pipeline.bind(cmd);

            self.camera_sets[current_frame as usize].bind(cmd, &self.pipeline, 0);
            self.sets[current_frame as usize].bind(cmd, &self.pipeline, 1);
            self.atlas_sets[current_frame as usize].bind(cmd, &self.pipeline, 2);

            self.mesh.draw_instanced(cmd, 0, self.triangles.len() as u32);
        }

        geometry_framebuffer.end_render_pass(cmd);

        // Clear all data
        self.triangles.clear();
    }

    pub fn add_shape(&mut self, triangle: InputTriangle) {
        if self.triangles.capacity() == self.triangles.len() {
            if self.triangles.len() as u64 == MAX_BATCH_SIZE {
                log::error!("Renderer2D: Maximum triangle draw limit exceeded");
                panic!();
            }
            let current_size = self.triangles.capacity();
            let new_size = (((current_size as f64) * BATCH_GROWTH_FACTOR) as usize)
                .min(MAX_BATCH_SIZE as usize);

            self.triangles.reserve_exact(new_size - self.triangles.len());

            self.device.wait_idle();

            for i in 0..self.buffers.len() {
                self.buffers[i] = Buffer::new(
                    self.device.clone(),
                    MVBufferCreateInfo {
                        instance_size: (new_size * size_of::<Triangle>()) as u64,
                        instance_count: 1,
                        buffer_usage: BufferUsage::STORAGE_BUFFER,
                        memory_properties: MemoryProperties::DEVICE_LOCAL,
                        minimum_alignment: 1,
                        memory_usage: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
                        label: Some("Renderer2D Triangle Buffer".to_string()),
                    },
                );

                self.sets[i].update_buffer(0, &self.buffers[i], 0, self.buffers[i].get_size())
            }
        }

        self.triangles.push(triangle.into());
    }

    // TODO: text
    pub fn text() -> ! {
        todo!()
        // Shape::Text {
        //     position,
        //     rotation,
        //     height,
        //     font_id,
        //     text,
        //     color,
        // } => {
        //     while self.rectangles.capacity() <= self.rectangles.len() + text.len() {
        //         if self.rectangles.len() as u64 == MAX_BATCH_SIZE {
        //             log::error!("Renderer2D: Maximum rectangle draw limit exceeded");
        //             panic!();
        //         }
        //         grow_batch!(
        //             self.rectangles,
        //             self.rectangle_buffers,
        //             self.rectangle_sets,
        //             Rectangle,
        //             "Rectangle"
        //         );
        //     }
        //
        //     if let Some(atlas) = self.font_data.get(&(font_id as u32)).cloned() {
        //         let mut font_scale = 1.0 / (atlas.metrics.ascender - atlas.metrics.descender);
        //         font_scale *= height as f64 / atlas.metrics.line_height;
        //         let space_advance = atlas.find_glyph(' ').unwrap().advance; // TODO
        //
        //         let mut x = 0.0;
        //         let mut y = 0.0;
        //
        //         for char in text.chars() {
        //             if (char == '\t') {
        //                 x += 6.0 + space_advance * font_scale;
        //                 continue;
        //             } else if (char == ' ') {
        //                 x += space_advance * font_scale;
        //                 continue;
        //             } else if (char == '\n') {
        //                 x = 0.0;
        //                 y -= font_scale * atlas.metrics.line_height;
        //                 continue;
        //             }
        //
        //             let glyph = if let Some(glyph) = atlas.find_glyph(char) {
        //                 glyph
        //             } else {
        //                 atlas.find_glyph('?').unwrap_or_else(|| {
        //                     log::error!("Font atlas missing 'missing character' glyph");
        //                     panic!()
        //                 })
        //             };
        //
        //             let bounds_plane = &glyph.plane_bounds;
        //             let bounds_atlas = &glyph.atlas_bounds;
        //
        //             let mut tex_coords = Vec4::new(
        //                 bounds_atlas.left as f32,
        //                 (atlas.atlas.height as f64 - bounds_atlas.top) as f32,
        //                 (bounds_atlas.right - bounds_atlas.left) as f32,
        //                 (bounds_atlas.top - bounds_atlas.bottom) as f32,
        //             );
        //
        //             tex_coords.x /= atlas.atlas.width as f32;
        //             tex_coords.y /= atlas.atlas.height as f32;
        //             tex_coords.z /= atlas.atlas.width as f32;
        //             tex_coords.w /= atlas.atlas.height as f32;
        //
        //             let mut scale = Vec2::new(
        //                 (bounds_plane.right - bounds_plane.left) as f32,
        //                 (bounds_plane.top - bounds_plane.bottom) as f32,
        //             );
        //             scale.x = scale.x * font_scale as f32;
        //             scale.y = scale.y * font_scale as f32;
        //
        //             let rot = rotation.to_radians();
        //
        //             let y_offset: f32 = (1.0 - bounds_plane.bottom) as f32 * scale.y;
        //
        //             // TODO: far plane is hardcoded to 100.0, change it. We use half the far plane to allow for negative z values
        //             let rectangle = Rectangle {
        //                 position: Vec4::new(
        //                     x as f32 + position.x,
        //                     y as f32 + position.y - y_offset,
        //                     50.0 - position.z,
        //                     0.0,
        //                 ),
        //                 rotation: Vec4::new(rot.x, rot.y, rot.z, 0.0),
        //                 scale,
        //                 tex_coord: tex_coords,
        //                 color,
        //                 tex_id: font_id as i32,
        //                 blending: -1.0,
        //             };
        //
        //             self.rectangles.push(rectangle);
        //
        //             x += glyph.advance * font_scale;
        //         }
        //     }
        // }
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
                label: Some(format!("Renderer2D {ty} Buffer")),
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
                label: Some(format!("Renderer2D {ty} Set")),
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
            include_str!("shaders/default.vert"),
            ShaderKind::Vertex,
            Some("Renderer2D Shared Vertex Shader".to_string()),
            &[],
        );
        let fragment_shader = self.core_renderer.get().compile_shader(
            include_str!("shaders/default.frag"),
            ShaderKind::Fragment,
            Some("Renderer2D Shared Fragment Shader".to_string()),
            &[],
        );

        self.pipeline = Pipeline::<Graphics>::new(
            self.device.clone(),
            MVGraphicsPipelineCreateInfo {
                shaders: vec![vertex_shader, fragment_shader],
                attributes: vec![],
                extent,
                topology: Topology::Triangle,
                cull_mode: CullMode::Back,
                enable_depth_test: true,
                depth_clamp: false,
                blending_enable: true,
                descriptor_sets: vec![
                    self.camera_sets[0].get_layout(),
                    self.sets[0].get_layout(),
                    self.atlas_sets[0].get_layout(),
                ],
                push_constants: vec![],
                framebuffer: self.geometry_framebuffers[0].clone(),
                color_attachments_count: 1,
                label: Some("Renderer2D Triangle Pipeline".to_string()),
            },
        );
    }
}
