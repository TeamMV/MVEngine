use log::LevelFilter;
use mvutils::version::Version;
use mvcore::math::vec::Vec4;
use mvcore::render::ApplicationLoopCallbacks;
use mvcore::render::backend::Backend;
use mvcore::render::backend::device::{Device, Extensions, MVDeviceCreateInfo};
use mvcore::render::window::{Window, WindowCreateInfo};
use mvengine_render2d::renderer2d::{Renderer2D, Transform};

fn main() {
    mvlogger::init(std::io::stdout(), LevelFilter::Debug);

    let window = Window::new(WindowCreateInfo{
        width: 800,
        height: 600,
        title: "TEST".to_string(),
        fullscreen: false,
        decorated: true,
        resizable: false,
        transparent: false,
        theme: None,
        vsync: false,
        max_frames_in_flight: 1,
        fps: 60,
        ups: 20,
    });

    window.run::<AppLoop>();
}

struct AppLoop {
    device: Device,
    renderer: Renderer2D,
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
            }, &window.get_handle());

        let renderer = Renderer2D::new(device.clone(), &window);

        Self {
            device,
            renderer,
        }
    }

    fn update(&mut self, window: &mut Window, delta_t: f64) {

    }

    fn draw(&mut self, window: &mut Window, delta_t: f64) {
        self.renderer.add_quad(Transform{
            position: Vec4::new(100.0, 100.0, 10.0, 0.0),
            rotation: Vec4::splat(0.0),
            scale: Vec4::splat(100.0),
        });
        self.renderer.draw();
    }

    fn exiting(&mut self, window: &mut Window) {
        self.device.wait_idle();
    }

    fn resize(&mut self, window: &mut Window, width: u32, height: u32) {

    }
}