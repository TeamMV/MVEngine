use crate::render::backend::device::{Device, Extensions, MVDeviceCreateInfo};
use crate::render::backend::Backend;
use log::LevelFilter;
use mvutils::version::Version;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

pub fn run() {
    mvlogger::init(std::io::stdout(), LevelFilter::Trace);

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_visible(true)
        .build(&event_loop)
        .unwrap();

    let device = Device::new(
        Backend::Vulkan,
        MVDeviceCreateInfo {
            app_name: "Test app".to_string(),
            app_version: Version::new(0, 1, 0),
            engine_name: "MVEngine".to_string(),
            engine_version: Version::new(0, 1, 0),
            device_extensions: Extensions::empty(),
        },
        &window,
    );

    event_loop
        .run(|event, target| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    WindowEvent::RedrawRequested => {}
                    _ => {}
                }
            }
        })
        .unwrap();
}
