use log::LevelFilter;
use mvcore::render::backend::device::{Device, Extensions, MVDeviceCreateInfo};
use mvcore::render::backend::Backend;
use mvcore::render::renderer::Renderer;
use mvcore::render::window::{Window, WindowCreateInfo};
use mvcore::render::ApplicationLoopCallbacks;
use mvcore::ToAD;
use mvutils::unsafe_utils::DangerousCell;
use mvutils::version::Version;
use std::sync::Arc;
use bytebuffer::ByteBuffer;
use mvutils::save::Savable;
use mvcore::color::parse::parse_color;
use mvcore::color::RgbColor;
use mvcore::render::backend::image::{AccessFlags, ImageLayout};
use mvcore::render::backend::sampler::Sampler;
use mvcore::render::font::{AtlasData, PreparedAtlasData};
use mvengine_render2d::gpu::Transform;
use mvengine_render2d::renderer2d::{InputTriangle, Renderer2D};

pub fn main() {
    mvlogger::init(std::io::stdout(), LevelFilter::Debug);

    let window = Window::new(WindowCreateInfo {
        width: 600,
        height: 600,
        title: "Renderer2D demo".to_string(),
        fullscreen: false,
        decorated: true,
        resizable: true,
        transparent: false,
        theme: None,
        vsync: false,
        max_frames_in_flight: 2,
        fps: 9999,
        ups: 20,
    });

    window.run::<AppLoop>();
}

pub struct AppLoop {
    device: Device,
    core_renderer: Arc<DangerousCell<Renderer>>,
    renderer2d: Arc<DangerousCell<Renderer2D>>,
    atlas_data: Arc<PreparedAtlasData>,
}

impl ApplicationLoopCallbacks for AppLoop {
    fn new(window: &mut Window) -> Self {
        let device = Device::new(
            Backend::Vulkan,
            MVDeviceCreateInfo {
                app_name: "Test app".to_string(),
                app_version: Version::new(0, 0, 1, 0),
                engine_name: "MVEngine".to_string(),
                engine_version: Version::new(0, 0, 1, 0),
                device_extensions: Extensions::empty(),
            },
            &window.get_handle(),
        );

        let core_renderer = Renderer::new(&window, device.clone()).to_ad();

        let renderer2d = Renderer2D::new(device.clone(), core_renderer.clone(), 0, 0).to_ad();

        let font_data_bytes = include_bytes!("data.font");
        let mut buffer = ByteBuffer::from_bytes(font_data_bytes);
        let atlas_data = Arc::new(
            AtlasData::load(&mut buffer)
                .unwrap_or_else(|err| {
                    log::error!("{err}");
                    panic!()
                })
                .into(),
        );
        drop(buffer);

        Self {
            device,
            core_renderer,
            renderer2d,
            atlas_data
        }
    }

    fn update(&mut self, window: &mut Window, delta_t: f64) {}

    fn draw(&mut self, window: &mut Window, delta_t: f64) {
        let renderer2d = self.renderer2d.get_mut();
        renderer2d.add_shape(InputTriangle {
            points: [(100, 100), (200, 100), (100, 200)],
            z: 1.0,
            transform: Transform::new(),
            canvas_transform: Transform::new(),
            tex_id: None,
            tex_coords: None,
            blending: 0.0,
            colors: [RgbColor::yellow().as_vec4(), RgbColor::cyan().as_vec4(), RgbColor::yellow().as_vec4()],
            is_font: false,
        });

        renderer2d.add_shape(InputTriangle {
            points: [(110, 100), (210, 100), (110, 200)],
            z: 1.1,
            transform: Transform::new(),
            canvas_transform: Transform::new(),
            tex_id: None,
            tex_coords: None,
            blending: 0.0,
            colors: [RgbColor::red().as_vec4(); 3],
            is_font: false,
        });

        renderer2d.add_shape(InputTriangle {
            points: [(400, 100), (500, 100), (400, 200)],
            z: 1.0,
            transform: Transform::new(),
            canvas_transform: Transform::new(),
            tex_id: None,
            tex_coords: None,
            blending: 0.0,
            colors: [RgbColor::blue().as_vec4(); 3],
            is_font: false,
        });

        let image_index = self.core_renderer.get_mut().begin_frame().unwrap();
        let cmd = self.core_renderer.get_mut().get_current_command_buffer();
        let frame_index = self.core_renderer.get().get_current_frame_index();

        renderer2d.draw();

        //u0407412

        cmd.blit_image(
            renderer2d.get_geometry_image(frame_index as usize),
            self.core_renderer
                .get_mut()
                .get_swapchain()
                .get_framebuffer(image_index as usize)
                .get_image(0),
        );

        self.core_renderer
            .get_mut()
            .get_swapchain()
            .get_framebuffer(image_index as usize)
            .get_image(0)
            .transition_layout(
                ImageLayout::PresentSrc,
                Some(cmd),
                AccessFlags::empty(),
                AccessFlags::empty(),
            );

        self.core_renderer.get_mut().end_frame().unwrap();
    }

    fn exiting(&mut self, window: &mut Window) {}

    fn resize(&mut self, window: &mut Window, width: u32, height: u32) {}
}
