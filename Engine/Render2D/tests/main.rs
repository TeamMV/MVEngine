use log::LevelFilter;
use mvcore::math::vec::{Vec2, Vec3, Vec4};
use mvcore::render::backend::device::{Device, Extensions, MVDeviceCreateInfo};
use mvcore::render::backend::Backend;
use mvcore::render::window::{Window, WindowCreateInfo};
use mvcore::render::ApplicationLoopCallbacks;
use mvengine_render2d::renderer2d::{Renderer2D, Transform};
use mvutils::version::Version;

fn main() {
    mvlogger::init(std::io::stdout(), LevelFilter::Debug);

    let window = Window::new(WindowCreateInfo {
        width: 600,
        height: 600,
        title: "Demo".to_string(),
        fullscreen: false,
        decorated: true,
        resizable: false,
        transparent: false,
        theme: None,
        vsync: false,
        max_frames_in_flight: 2,
        fps: 9999,
        ups: 20,
    });

    window.run::<AppLoop>();
}

struct AppLoop {
    device: Device,
    renderer: Renderer2D,

    quad_rotation: f32,
    quad_position: Vec2,
    timer: f32
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

        let renderer = Renderer2D::new(device.clone(), &window);

        Self { device, renderer, quad_rotation: 0.0, quad_position: Vec2::splat(0.0), timer: 0.0 }
    }

    fn update(&mut self, window: &mut Window, delta_t: f64) {}

    fn draw(&mut self, window: &mut Window, delta_t: f64) {
        log::info!("ms: {}. FPS {}", delta_t * 1000.0, 1.0 / delta_t);

        self.timer += delta_t as f32;
        self.quad_rotation += delta_t as f32 * 2.0;

        self.quad_position.x = self.timer.sin() * 100.0;
        self.quad_position.y = self.timer.cos() * 100.0;

        self.renderer.add_quad(Transform {
            position: Vec3::new(self.quad_position.x + 300.0, -self.quad_position.y + 400.0, 1.0),
            rotation: self.quad_rotation,
            scale: Vec2::splat(50.0),
        });

        self.renderer.add_quad(Transform {
            position: Vec3::new(self.quad_position.x + 300.0, -self.quad_position.y + 200.0, 1.0),
            rotation: -self.quad_rotation,
            scale: Vec2::splat(50.0),
        });

        self.renderer.add_quad(Transform {
            position: Vec3::new(self.quad_position.x + 200.0, -self.quad_position.y + 300.0, 1.0),
            rotation: self.quad_rotation,
            scale: Vec2::splat(50.0),
        });

        self.renderer.add_quad(Transform {
            position: Vec3::new(self.quad_position.x + 400.0, -self.quad_position.y + 300.0, 1.0),
            rotation: -self.quad_rotation,
            scale: Vec2::splat(50.0),
        });

        self.renderer.draw();
    }

    fn exiting(&mut self, window: &mut Window)
    {
        self.device.wait_idle();
    }

    fn resize(&mut self, window: &mut Window, width: u32, height: u32) {}
}
