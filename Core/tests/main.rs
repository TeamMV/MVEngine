use std::sync::Arc;
use log::LevelFilter;
use mvutils::version::Version;
use mvcore::asset::asset::AssetType;
use mvcore::asset::manager::{AssetHandle, AssetManager};
use mvcore::math::vec::Vec4;
use mvcore::render::ApplicationLoopCallbacks;
use mvcore::render::backend::Backend;
use mvcore::render::backend::device::{Device, Extensions, MVDeviceCreateInfo};
use mvcore::render::window::{Window, WindowCreateInfo};

fn main() {
    mvlogger::init(std::io::stdout(), LevelFilter::Debug);
    mvcore::err::setup();

    let window = Window::new(WindowCreateInfo {
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
    manager: Arc<AssetManager>,
    handle: AssetHandle,
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

        let manager = AssetManager::new(device.clone(), 1);

        let handle = manager.create_asset("texture.png", AssetType::Texture);

        handle.load();

        Self { device, manager, handle }
    }

    fn update(&mut self, window: &mut Window, delta_t: f64) {}

    fn draw(&mut self, window: &mut Window, delta_t: f64) {
        let texture = self.handle.get();
        if texture.failed() {
            //draw failed thing
            println!("Failed")
        } else if !texture.is_loaded() {
            //draw loading thing
            println!("loading")
        } else {
            let Some(texture) = texture.as_texture() else { unreachable!() };
            //draw the texture ig
            println!("loaded")
        }
    }

    fn exiting(&mut self, window: &mut Window) {}

    fn resize(&mut self, window: &mut Window, width: u32, height: u32) {}
}
