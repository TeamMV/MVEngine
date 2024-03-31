use log::LevelFilter;
use mvutils::once::CreateOnce;
use shaderc::ShaderKind;
use mvcore::render::window::{Window, WindowCreateInfo};
use mvcore::render::ApplicationLoopCallbacks;
use mvcore::render::backend::{Backend, Extent2D};
use mvcore::render::backend::device::{Device, Extensions, MVDeviceCreateInfo};
use mvcore::render::backend::framebuffer::ClearColor;
use mvcore::render::backend::pipeline::{AttributeType, CullMode, Graphics, MVGraphicsPipelineCreateInfo, Pipeline, Topology};
use mvcore::render::backend::shader::ShaderStage;
use mvcore::render::mesh2d::Mesh;
use mvcore::render::renderer::Renderer;

fn main() {
    mvlogger::init(std::io::stdout(), LevelFilter::Debug);

    let window = Window::new(WindowCreateInfo {
        width: 800,
        height: 600,
        title: "MVCore".to_string(),
        fullscreen: false,
        decorated: true,
        resizable: true,
        transparent: false,
        theme: None,
        vsync: false,
        max_frames_in_flight: 2,
        fps: 60,
        ups: 20,
    });

    window.run::<ApplicationLoop>();
}

struct ApplicationLoop {
    device: Device,
    mesh: Mesh,
    pipeline: Pipeline,
    renderer: Renderer
}

#[repr(C)]
struct Vertex {
    position: [f32; 3],
    tex_coord: [f32; 2]
}

impl Vertex {
    pub fn to_bytes(vertices: &[Vertex]) -> &[u8] {
        unsafe { std::slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices.len() * std::mem::size_of::<Vertex>()) }
    }

    pub fn get_attribute_description() -> Vec<AttributeType> {
        vec![AttributeType::Float32x3, AttributeType::Float32x2]
    }
}

impl ApplicationLoopCallbacks for ApplicationLoop {
    fn new(window: &mut Window) -> Self {
        let device = Device::new(
            Backend::Vulkan, MVDeviceCreateInfo {
                app_name: "".to_string(),
                app_version: Default::default(),
                engine_name: "".to_string(),
                engine_version: Default::default(),
                device_extensions: Extensions::empty(),
            },
            window.get_handle()
        );

        let renderer = Renderer::new(window, device.clone());

        let vertices = vec![
            Vertex{position: [ 0.0f32, -0.5, 0.0], tex_coord: [0.0f32, 0.0]},
            Vertex{position: [ 0.5f32,  0.5, 0.0], tex_coord: [0.0f32, 0.0]},
            Vertex{position: [-0.5f32,  0.5, 0.0], tex_coord: [0.0f32, 0.0]},
        ];

        let indices = [0, 1, 2];

        let mesh = Mesh::new(device.clone(), Vertex::to_bytes(&vertices), Some(&indices), Some("Test mesh".to_string()));

        let vertex_shader = renderer.compile_shader(include_str!("../src/render/shaders/2d/default.vert"), ShaderStage::Vertex, ShaderKind::Vertex);
        let fragment_shader = renderer.compile_shader(include_str!("../src/render/shaders/2d/default.frag"), ShaderStage::Fragment, ShaderKind::Fragment);

        let test_pipeline = Pipeline::<Graphics>::new(device.clone(), MVGraphicsPipelineCreateInfo {
            shaders: vec![vertex_shader, fragment_shader],
            attributes: Vertex::get_attribute_description(),
            extent: renderer.get_swapchain().get_extent(),
            topology: Topology::Triangle,
            cull_mode: CullMode::None,
            enable_depth_test: false,
            depth_clamp: false,
            blending_enable: false,
            descriptor_sets: vec![],
            push_constants: vec![],
            framebuffer: renderer.get_swapchain().get_framebuffer(0),
            color_attachments_count: 1,
            label: None,
        });

        ApplicationLoop {
            mesh,
            device,
            pipeline: test_pipeline,
            renderer
        }
    }

    fn update(&mut self, window: &mut Window, delta_t: f64) {
        println!("update: {delta_t}");
    }

    fn draw(&mut self, window: &mut Window, delta_t: f64) {
        let image_index = self.renderer.begin_frame().unwrap();

        let swapchain = self.renderer.get_swapchain_mut();
        let extent = swapchain.get_extent();
        let framebuffer = swapchain.get_current_framebuffer();
        let cmd = self.renderer.get_current_command_buffer();

        framebuffer.begin_render_pass(cmd, &[ClearColor::Color([0.0, 0.0, 0.0, 1.0])], extent);

        self.pipeline.bind(cmd);

        self.mesh.draw(cmd);

        framebuffer.end_render_pass(cmd);

        self.renderer.end_frame().unwrap();
    }

    fn exiting(&mut self, window: &mut Window) {
        println!("exit");
    }

    fn resize(&mut self, window: &mut Window, width: u32, height: u32) {
        println!("resize {width} {height}");
    }
}
