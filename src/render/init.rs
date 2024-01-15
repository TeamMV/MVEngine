use std::cmp::min;
use std::num::NonZeroU32;
use std::sync::Arc;

use itertools::Itertools;
use mvsync::block::AwaitSync;
use mvutils::utils::TetrahedronOp;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{AddressMode, Backend, Backends, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendComponent, BlendFactor, BlendOperation, BlendState, Buffer, BufferDescriptor, BufferUsages, ColorWrites, CompositeAlphaMode, DepthStencilState, Device, DeviceDescriptor, Extent3d, Face, FilterMode, FragmentState, FrontFace, IndexFormat, InstanceDescriptor, PolygonMode, PowerPreference, PresentMode, PrimitiveState, PrimitiveTopology, Queue, RenderPipeline, RequestAdapterOptions, SamplerDescriptor, ShaderModule, ShaderStages, StencilState, Surface, SurfaceConfiguration, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension, VertexBufferLayout, VertexState};
use wgpu::{Instance, InstanceFlags};
use winit::dpi::PhysicalSize;

use crate::render::common::Texture;
use crate::render::consts::{BIND_GROUPS, BIND_GROUP_2D, BIND_GROUP_BATCH_3D, BIND_GROUP_EFFECT, BIND_GROUP_EFFECT_CUSTOM, BIND_GROUP_GEOMETRY_3D, BIND_GROUP_LAYOUT_2D, BIND_GROUP_LAYOUT_EFFECT, BIND_GROUP_LAYOUT_EFFECT_CUSTOM, BIND_GROUP_LAYOUT_GEOMETRY_3D, BIND_GROUP_LAYOUT_LIGHTING_3D, BIND_GROUP_LAYOUT_3D, BIND_GROUP_LAYOUT_MODEL_MATRIX, BIND_GROUP_LIGHTING_3D, BIND_GROUP_MODEL_3D, BIND_GROUP_MODEL_MATRIX, BIND_GROUP_TEXTURES, DEFAULT_SAMPLER, DUMMY_TEXTURE, INDEX_LIMIT, LIGHT_LIMIT, MATERIAL_LIMIT, MAX_LIGHTS, MAX_TEXTURES, TEXTURE_LIMIT, VERTEX_LAYOUT_2D, VERTEX_LAYOUT_3D, VERTEX_LAYOUT_NONE, VERT_LIMIT_2D_BYTES, MAX_MATERIALS};
use crate::render::window::WindowSpecs;

pub(crate) struct State {
    pub(crate) surface: Surface,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) config: SurfaceConfiguration,
    pub(crate) backend: Backend,
}

impl State {
    pub(crate) fn new(window: &winit::window::Window, specs: &WindowSpecs) -> Self {
        Self::init(window, specs).await_sync()
    }

    async fn init(window: &winit::window::Window, specs: &WindowSpecs) -> Self {
        unsafe {
            let instance = Instance::new(InstanceDescriptor {
                backends: Backends::GL
                    | Backends::VULKAN
                    | Backends::DX12
                    | Backends::METAL,
                flags: InstanceFlags::from_build_config(),
                dx12_shader_compiler: Default::default(),
                gles_minor_version: Default::default(),
            });

            let surface = instance
                .create_surface(window)
                .expect("Could not create window surface!");

            let adapter = instance.request_adapter(
                &RequestAdapterOptions {
                    power_preference: specs.green_eco_mode.yn(PowerPreference::LowPower, PowerPreference::HighPerformance),
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                }
            ).await.expect("Graphical adapter cannot be found! (This is usually a driver issue, or you are missing hardware)");

            let textures = adapter.limits().max_sampled_textures_per_shader_stage;

            let _ = MAX_TEXTURES.try_create(|| min(textures as usize - 1, TEXTURE_LIMIT));

            let _ = MAX_LIGHTS.try_create(|| LIGHT_LIMIT);

            let _ = MAX_MATERIALS.try_create(|| MATERIAL_LIMIT);

            let (device, queue) = adapter
                .request_device(
                    &DeviceDescriptor {
                        features: adapter.features(),
                        limits: adapter.limits(),
                        label: Some("GPU"),
                    },
                    None,
                )
                .await
                .expect("Could not create logical device!");

            let surface_caps = surface.get_capabilities(&adapter);
            let surface_format = surface_caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(surface_caps.formats[0]);

            let surface_alpha = surface_caps
                .alpha_modes
                .contains(&CompositeAlphaMode::Opaque)
                .yn(CompositeAlphaMode::Opaque, surface_caps.alpha_modes[0]);

            let config = SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: specs.width,
                height: specs.height,
                present_mode: specs
                    .vsync
                    .yn(PresentMode::AutoVsync, PresentMode::AutoNoVsync),
                alpha_mode: surface_alpha,
                view_formats: vec![],
            };
            surface.configure(&device, &config);

            let _ = BIND_GROUPS.try_init(|groups| {
                groups.insert(
                    BIND_GROUP_2D,
                    device.create_bind_group_layout(&BIND_GROUP_LAYOUT_2D),
                );
                groups.insert(
                    BIND_GROUP_TEXTURES,
                    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: Some("Bind group layout textures 2D"),
                        entries: &[BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                multisampled: false,
                                view_dimension: TextureViewDimension::D2,
                                sample_type: TextureSampleType::Float { filterable: true },
                            },
                            count: Some(NonZeroU32::new_unchecked(*MAX_TEXTURES as u32)),
                        }],
                    }),
                );
                groups.insert(
                    BIND_GROUP_MODEL_MATRIX,
                    device.create_bind_group_layout(&BIND_GROUP_LAYOUT_MODEL_MATRIX),
                );
                groups.insert(
                    BIND_GROUP_MODEL_3D,
                    device.create_bind_group_layout(&BIND_GROUP_LAYOUT_3D),
                );
                groups.insert(
                    BIND_GROUP_GEOMETRY_3D,
                    device.create_bind_group_layout(&BIND_GROUP_LAYOUT_GEOMETRY_3D),
                );
                groups.insert(
                    BIND_GROUP_LIGHTING_3D,
                    device.create_bind_group_layout(&BIND_GROUP_LAYOUT_LIGHTING_3D),
                );
                groups.insert(
                    BIND_GROUP_EFFECT,
                    device.create_bind_group_layout(&BIND_GROUP_LAYOUT_EFFECT),
                );
                groups.insert(
                    BIND_GROUP_EFFECT_CUSTOM,
                    device.create_bind_group_layout(&BIND_GROUP_LAYOUT_EFFECT_CUSTOM),
                );
            });

            let texture = device.create_texture(&TextureDescriptor {
                label: Some("Dummy texture"),
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let view = texture.create_view(&TextureViewDescriptor::default());

            let _ = DUMMY_TEXTURE.try_create(|| Arc::new(Texture::premade(texture, view)));

            let _ = DEFAULT_SAMPLER.try_create(|| {
                device.create_sampler(&SamplerDescriptor {
                    label: Some("Texture sampler"),
                    address_mode_u: AddressMode::Repeat,
                    address_mode_v: AddressMode::Repeat,
                    address_mode_w: AddressMode::Repeat,
                    mag_filter: FilterMode::Linear,
                    min_filter: FilterMode::Linear,
                    mipmap_filter: FilterMode::Linear,
                    lod_min_clamp: 0.0,
                    lod_max_clamp: 32.0,
                    compare: None,
                    anisotropy_clamp: 1,
                    border_color: None,
                })
            });

            Self {
                surface,
                device,
                queue,
                config,
                backend: adapter.get_info().backend,
            }
        }
    }

    pub(crate) fn gen_buffers(&self) -> (Buffer, Buffer) {
        (self.gen_vbo(), self.gen_ibo())
    }

    pub(crate) fn gen_buffers_sized(&self, vbo: u64, ibo: u64) -> (Buffer, Buffer) {
        (self.gen_vbo_sized(vbo), self.gen_ibo_sized(ibo))
    }

    pub(crate) fn gen_vbo(&self) -> Buffer {
        self.device.create_buffer(&BufferDescriptor {
            label: Some("vbo"),
            size: VERT_LIMIT_2D_BYTES,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub(crate) fn gen_vbo_sized(&self, size: u64) -> Buffer {
        self.device.create_buffer(&BufferDescriptor {
            label: Some("vbo"),
            size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub(crate) fn gen_ibo(&self) -> Buffer {
        self.device.create_buffer(&BufferDescriptor {
            label: Some("ibo"),
            size: INDEX_LIMIT,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub(crate) fn gen_ibo_sized(&self, size: u64) -> Buffer {
        self.device.create_buffer(&BufferDescriptor {
            label: Some("ibo"),
            size,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub(crate) fn gen_uniform_buffer(&self, data: &[u8]) -> Buffer {
        self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("buffer"),
            contents: data,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        })
    }

    pub(crate) fn gen_uniform_buffer_sized(&self, size: u64) -> Buffer {
        self.device.create_buffer(&BufferDescriptor {
            label: Some("buffer"),
            size,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        })
    }

    fn create_render_pipeline(
        &self,
        vertex_shader: &ShaderModule,
        fragment_shader: Option<&ShaderModule>,
        render_mode: PrimitiveTopology,
        cull_dir: FrontFace,
        cull_mode: Face,
        pol_mode: PolygonMode,
        vertex_layout: VertexBufferLayout,
        bind_groups: Vec<&'static BindGroupLayout>,
    ) -> RenderPipeline {
        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &bind_groups,
                    push_constant_ranges: &[],
                });

        let strip_index_format = render_mode.is_strip().yn(Some(IndexFormat::Uint32), None);

        let stencil_state = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Always,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::IncrementClamp,
        };


        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: VertexState {
                    module: vertex_shader,
                    entry_point: fragment_shader.is_none().yn("vert", "main"),
                    buffers: &[vertex_layout],
                },
                fragment: Some(FragmentState {
                    module: fragment_shader.unwrap_or(vertex_shader),
                    entry_point: fragment_shader.is_none().yn("frag", "main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.config.format,
                        blend: Some(BlendState {
                            color: BlendComponent {
                                src_factor: BlendFactor::SrcAlpha,
                                dst_factor: BlendFactor::OneMinusSrcAlpha,
                                operation: BlendOperation::Add,
                            },
                            alpha: BlendComponent {
                                src_factor: BlendFactor::One,
                                dst_factor: BlendFactor::OneMinusSrcAlpha,
                                operation: BlendOperation::Add,
                            },
                        }),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    topology: render_mode,
                    strip_index_format,
                    front_face: cull_dir,
                    cull_mode: Some(cull_mode),
                    polygon_mode: pol_mode,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            })
    }

    pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub(crate) fn vsync(&mut self, vsync: bool) {
        self.config.present_mode = vsync.yn(PresentMode::AutoVsync, PresentMode::AutoNoVsync);
        self.surface.configure(&self.device, &self.config);
    }
}

pub(crate) struct PipelineBuilder<'a> {
    state: &'a State,

    vertex_layout: VertexBufferLayout<'static>,
    bind_group: Vec<u8>,
    vert: Option<&'a ShaderModule>,
    frag: Option<&'a ShaderModule>,
    render_mode: PrimitiveTopology,
    cull_direction: FrontFace,
    cull_mode: Face,
    polygon_mode: PolygonMode,
}

impl<'a> PipelineBuilder<'a> {
    pub(crate) const RENDER_MODE: u8 = 0;
    pub(crate) const CULL_DIR: u8 = 1;
    pub(crate) const CULL_MODE: u8 = 2;
    pub(crate) const POLYGON_MODE: u8 = 3;
    pub(crate) const SHADER_VERTEX: u8 = 4;
    pub(crate) const SHADER_FRAGMENT: u8 = 5;
    pub(crate) const SHADER_COMMON: u8 = 6;
    pub(crate) const VERTEX_LAYOUT: u8 = 7;
    pub(crate) const BIND_GROUP: u8 = 8;

    pub(crate) const CULL_DIR_CLOCKWISE: u8 = 10;
    pub(crate) const CULL_DIR_COUNTERCLOCKWISE: u8 = 11;
    pub(crate) const CULL_MODE_FORWARD: u8 = 12;
    pub(crate) const CULL_MODE_BACKWARDS: u8 = 13;
    pub(crate) const RENDER_MODE_TRIANGLES: u8 = 14;
    pub(crate) const RENDER_MODE_LINES: u8 = 15;
    pub(crate) const RENDER_MODE_POINTS: u8 = 16;
    pub(crate) const RENDER_MODE_TRIANGLE_STRIP: u8 = 17;
    pub(crate) const RENDER_MODE_LINE_STRIP: u8 = 18;
    pub(crate) const POLYGON_MODE_FILL: u8 = 19;
    pub(crate) const POLYGON_MODE_LINE: u8 = 20;
    pub(crate) const POLYGON_MODE_POINT: u8 = 21;
    pub(crate) const VERTEX_LAYOUT_2D: u8 = 22;
    pub(crate) const VERTEX_LAYOUT_MODEL_3D: u8 = 23;
    pub(crate) const VERTEX_LAYOUT_BATCH_3D: u8 = 24;
    pub(crate) const VERTEX_LAYOUT_NONE: u8 = 25;

    pub(crate) fn begin(state: &'a State) -> Self {
        Self {
            state,
            vertex_layout: VERTEX_LAYOUT_2D,
            bind_group: Vec::new(),
            vert: None,
            frag: None,
            render_mode: PrimitiveTopology::TriangleList,
            cull_direction: FrontFace::Ccw,
            cull_mode: Face::Back,
            polygon_mode: PolygonMode::Fill,
        }
    }

    fn get_cull_dir(&self, n: u8) -> FrontFace {
        match n {
            Self::CULL_DIR_CLOCKWISE => FrontFace::Cw,
            Self::CULL_DIR_COUNTERCLOCKWISE => FrontFace::Ccw,
            _ => FrontFace::Ccw,
        }
    }

    fn get_cull_mode(&self, n: u8) -> Face {
        match n {
            Self::CULL_MODE_BACKWARDS => Face::Back,
            Self::CULL_MODE_FORWARD => Face::Front,
            _ => Face::Back,
        }
    }

    fn get_pol_mode(&self, n: u8) -> PolygonMode {
        match n {
            Self::POLYGON_MODE_FILL => PolygonMode::Fill,
            Self::POLYGON_MODE_LINE => PolygonMode::Line,
            Self::POLYGON_MODE_POINT => PolygonMode::Point,
            _ => PolygonMode::Fill,
        }
    }

    fn get_ren_mode(&self, n: u8) -> PrimitiveTopology {
        match n {
            Self::RENDER_MODE_TRIANGLES => PrimitiveTopology::TriangleList,
            Self::RENDER_MODE_TRIANGLE_STRIP => PrimitiveTopology::TriangleStrip,
            Self::RENDER_MODE_LINES => PrimitiveTopology::LineList,
            Self::RENDER_MODE_LINE_STRIP => PrimitiveTopology::LineStrip,
            Self::RENDER_MODE_POINTS => PrimitiveTopology::PointList,
            _ => PrimitiveTopology::TriangleList,
        }
    }

    pub(crate) fn param(mut self, which: u8, what: u8) -> Self {
        match which {
            Self::RENDER_MODE => self.render_mode = self.get_ren_mode(what),
            Self::POLYGON_MODE => self.polygon_mode = self.get_pol_mode(what),
            Self::CULL_DIR => self.cull_direction = self.get_cull_dir(what),
            Self::CULL_MODE => self.cull_mode = self.get_cull_mode(what),
            Self::VERTEX_LAYOUT => match what {
                Self::VERTEX_LAYOUT_2D => self.vertex_layout = VERTEX_LAYOUT_2D,
                Self::VERTEX_LAYOUT_MODEL_3D => self.vertex_layout = VERTEX_LAYOUT_MODEL_3D,
                Self::VERTEX_LAYOUT_BATCH_3D => self.vertex_layout = VERTEX_LAYOUT_3D,
                Self::VERTEX_LAYOUT_NONE => self.vertex_layout = VERTEX_LAYOUT_NONE,
                _ => {}
            },
            Self::BIND_GROUP => self.bind_group.push(what),
            _ => {}
        }

        self
    }

    pub(crate) fn custom_vertex_layout(mut self, layout: VertexBufferLayout<'static>) -> Self {
        self.vertex_layout = layout;
        self
    }

    pub(crate) fn shader(mut self, which: u8, shader: &'a ShaderModule) -> Self {
        if which == PipelineBuilder::SHADER_VERTEX || which == PipelineBuilder::SHADER_COMMON {
            self.vert = Some(shader);
        }
        if which == PipelineBuilder::SHADER_FRAGMENT {
            self.frag = Some(shader);
        }
        self
    }

    pub(crate) fn bind_groups(mut self, groups: &[u8]) -> Self {
        self.bind_group = groups.to_vec();
        self
    }

    pub(crate) fn build(self) -> RenderPipeline {
        if self.vert.is_none() {
            panic!("Vertex/Common shader can't be None when creating pipeline!");
        }

        let bindings = self
            .bind_group
            .into_iter()
            .map(|b| BIND_GROUPS.get(&b).expect("Illegal bind group id!"))
            .collect_vec();

        self.state.create_render_pipeline(
            self.vert.unwrap(),
            self.frag,
            self.render_mode,
            self.cull_direction,
            self.cull_mode,
            self.polygon_mode,
            self.vertex_layout,
            bindings,
        )
    }
}
