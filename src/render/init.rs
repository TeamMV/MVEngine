use std::num::{NonZeroI16, NonZeroU32};
use std::ops::Deref;
use mvutils::utils::TetrahedronOp;
use naga::ShaderStage;
use tokio::runtime::Runtime;
use wgpu::{Queue, Surface, Device, SurfaceConfiguration, InstanceDescriptor, PowerPreference, Backends, Backend, RequestAdapterOptions, DeviceDescriptor, Features, Limits, TextureUsages, PresentMode, CompositeAlphaMode, RenderPipeline, ShaderModuleDescriptor, ShaderSource, ShaderModule, PrimitiveTopology, PolygonMode, FrontFace, Face, IndexFormat, DepthStencilState, VertexState, FragmentState, PrimitiveState};
use wgpu::Instance;
use winit::dpi::PhysicalSize;
use crate::render::window::{Window, WindowSpecs};

pub(crate) struct State {
    pub(crate) surface: Surface,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) config: SurfaceConfiguration,
    pub(crate) render_pipeline: RenderPipeline,
}

impl State {
    pub(crate) fn new(window: &winit::window::Window, specs: &WindowSpecs) -> Self {
        let rt = Runtime::new().expect("Could not create State due to async function faliure!");
        rt.block_on(Self::init(window, specs))
    }

    async fn init(window: &winit::window::Window, specs: &WindowSpecs) -> Self {
        unsafe {
            let instance = Instance::new(InstanceDescriptor {
                backends: Backends::GL
                    | Backends::VULKAN
                    | Backends::DX11
                    | Backends::DX12
                    | Backends::METAL,
                dx12_shader_compiler: Default::default(),
            });

            let surface = instance.create_surface(window).expect("Could not create window surface!");

            let adapter = instance.request_adapter(
                &RequestAdapterOptions {
                    power_preference: specs.green_eco_mode.yn(PowerPreference::LowPower, PowerPreference::HighPerformance),
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                }
            ).await.expect("Graphical adapter cannot be found for this window! (This is usually a driver issue, or you are missing hardware)");

            /*
            let adapter = instance
                .enumerate_adapters(wgpu::Backends::all())
                .filter(|adapter| {
                    // Check if this adapter supports our surface
                    adapter.is_surface_supported(&surface)
                })
                .next()
                .unwrap()
             */

            let (device, queue) = adapter.request_device(
                &DeviceDescriptor {
                    features: adapter.features(),
                    limits: adapter.limits(),
                    label: Some("GPU"),
                },
                None
            ).await.expect("Could not create logical device!");

            let surface_caps = surface.get_capabilities(&adapter);
            let surface_format = surface_caps.formats.iter()
                .copied()
                .filter(|f| f.is_srgb())
                .next()
                .unwrap_or(surface_caps.formats[0]);

            let surface_alpha = surface_caps.alpha_modes.contains(&CompositeAlphaMode::Opaque).yn(
                CompositeAlphaMode::Opaque,
                surface_caps.alpha_modes[0]
            );

            let config = SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: specs.width,
                height: specs.height,
                present_mode: specs.vsync.yn(PresentMode::AutoVsync, PresentMode::AutoNoVsync),
                alpha_mode: surface_alpha,
                view_formats: vec![],
            };
            surface.configure(&device, &config);

            let vert = device.create_shader_module(ShaderModuleDescriptor {
                label: Some("vert"),
               source: ShaderSource::Glsl {
                   shader: include_str!("shader.vert").into(),
                   stage: ShaderStage::Vertex,
                   defines: Default::default(),
               }
            });

            let frag = device.create_shader_module(ShaderModuleDescriptor {
                label: Some("frag"),
                source: ShaderSource::Glsl {
                    shader: include_str!("shader.frag").into(),
                    stage: ShaderStage::Fragment,
                    defines: Default::default(),
                }
            });

            let render_pipeline = PipelineBuilder::begin_using(&device, &config)
                .shader(PipelineBuilder::SHADER_VERTEX, &vert)
                .shader(PipelineBuilder::SHADER_FRAGMENT, &frag)
                .build();

            Self {
                surface,
                device,
                queue,
                config,
                render_pipeline
            }
        }
    }

    fn create_render_pipeline(device: &Device, config: &SurfaceConfiguration, vertex_shader: &ShaderModule, fragment_shader: Option<&ShaderModule>, render_mode: PrimitiveTopology, cull_dir: FrontFace, cull_mode: Face, pol_mode: PolygonMode) -> RenderPipeline {
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let strip_index_format = render_mode.is_strip().yn(Some(IndexFormat::Uint32), None);

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: vertex_shader,
                entry_point: fragment_shader.is_none().yn("vert", "main"),
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: fragment_shader.unwrap_or(vertex_shader),
                entry_point: fragment_shader.is_none().yn("frag", "main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
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
    device: &'a Device,
    config: &'a SurfaceConfiguration,

    vert: Option<&'a ShaderModule>,
    frag: Option<&'a ShaderModule>,
    render_mode: PrimitiveTopology,
    cull_direction: FrontFace,
    cull_mode: Face,
    polygon_mode: PolygonMode,
}

impl<'a> PipelineBuilder<'a> {
    const RENDER_MODE: u8 = 0;
    const CULL_DIR: u8 = 1;
    const CULL_MODE: u8 = 2;
    const POLYGON_MODE: u8 = 3;
    const SHADER_VERTEX: u8 = 4;
    const SHADER_FRAGMENT: u8 = 5;
    const SHADER_COMMON: u8 = 6;

    const CULL_DIR_CLOCKWISE: u8 = 10;
    const CULL_DIR_COUNTERCLOCKWISE: u8 = 11;
    const CULL_MODE_FORWARD: u8 = 12;
    const CULL_MODE_BACKWARDS: u8 = 13;
    const RENDER_MODE_TRIANGLES: u8 = 14;
    const RENDER_MODE_LINES: u8 = 15;
    const RENDER_MODE_POINTS: u8 = 16;
    const RENDER_MODE_TRIANGLE_STRIP: u8 = 17;
    const RENDER_MODE_LINE_STRIP: u8 = 18;
    const POLYGON_MODE_FILL: u8 = 19;
    const POLYGON_MODE_LINE: u8 = 20;
    const POLYGON_MODE_POINT: u8 = 21;

    pub(crate) fn begin(state: &'a State) -> Self {
        Self {
            device: &state.device,
            config: &state.config,
            vert: None,
            frag: None,
            render_mode: PrimitiveTopology::TriangleList,
            cull_direction: FrontFace::Ccw,
            cull_mode: Face::Back,
            polygon_mode: PolygonMode::Fill,
        }
    }

    pub(crate) fn begin_using(device: &'a Device, config: &'a SurfaceConfiguration) -> Self {
        Self {
            device,
            config,
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
            Self::RENDER_MODE => {self.render_mode = self.get_ren_mode(what)}
            Self::POLYGON_MODE => {self.polygon_mode = self.get_pol_mode(what)}
            Self::CULL_DIR => {self.cull_direction = self.get_cull_dir(what)}
            Self::CULL_MODE => {self.cull_mode = self.get_cull_mode(what)}
            _ => {}
        }

        self
    }

    pub(crate) fn shader(mut self, which: u8, shader: &'a ShaderModule) -> Self {
        if which == PipelineBuilder::SHADER_VERTEX || which == PipelineBuilder::SHADER_COMMON {self.vert = Some(shader);}
        if which == PipelineBuilder::SHADER_FRAGMENT {self.frag = Some(shader);}
        self
    }

    pub(crate) fn build(self) -> RenderPipeline {
        if self.vert.is_none() {
            panic!("Vertex/Common can't be None when creating pipeline!");
        }

        State::create_render_pipeline(self.device, self.config, self.vert.unwrap(), self.frag, self.render_mode, self.cull_direction, self.cull_mode, self.polygon_mode)
    }
}