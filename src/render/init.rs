use mvutils::utils::TetrahedronOp;
use tokio::runtime::Runtime;
use wgpu::{Queue, Surface, Device, SurfaceConfiguration, InstanceDescriptor, PowerPreference, Backends, Backend, RequestAdapterOptions, DeviceDescriptor, Features, Limits, TextureUsages, PresentMode, CompositeAlphaMode};
use wgpu::Instance;
use winit::dpi::PhysicalSize;
use crate::render::window::{Window, WindowSpecs};

pub(crate) struct State {
    pub(crate) surface: Surface,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) config: SurfaceConfiguration,
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

            Self {
                surface,
                device,
                queue,
                config
            }
        }
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