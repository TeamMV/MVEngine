use include_dir::include_dir;
use log::LevelFilter;
use mvengine::rendering::api::{
    Renderer, RendererCreateInfo, RendererCreateInfoFlags, RendererFlavor,
};
use mvengine::rendering::loading::obj::OBJModelLoader;
use mvengine::window::app::WindowCallbacks;
use mvengine::window::{Error, Window, WindowCreateInfo};
use mvengine_ui_parsing::json::{JsonIdentFlavor, parse_json};
use mvutils::once::CreateOnce;
use mvutils::utils::setup_private_panic;
use mvutils::version::Version;
use parking_lot::RwLock;
use std::io::stdout;
use std::sync::Arc;

pub fn main() -> Result<(), Error> {
    mvengine::panic::setup_logger(stdout(), LevelFilter::Trace, 1000);
    mvengine::panic::setup_panic(true, "mve/logs");

    let mut info = WindowCreateInfo::default();
    info.title = "Window demo".to_string();
    info.fps = 60;
    info.ups = 20;
    info.vsync = true;

    let window = Window::new(info);
    let arc = Arc::new(RwLock::new(Application::new()));
    window.run::<Application>(arc)
}

struct Application {
    renderer: CreateOnce<Renderer>,
}

impl Application {
    fn new() -> Self {
        Self {
            renderer: CreateOnce::new(),
        }
    }
}

impl WindowCallbacks for Application {
    fn post_init(&mut self, window: &mut Window) {
        // let mut renderer = Renderer::new(
        //     window,
        //     RendererCreateInfo {
        //         app_name: "Hello Vulkan".to_string(),
        //         version: Default::default(),
        //         flags: RendererCreateInfoFlags::VSYNC,
        //         flavor: RendererFlavor::FLAVOR_3D,
        //         frames_in_flight: 2,
        //     },
        // );

        let raw_json = "{hello: 4, lmao: [\"1\", {}, []]}";
        let json = parse_json(raw_json, JsonIdentFlavor::UnquotedIdents).unwrap();
        let o = json.as_object().unwrap();
        o.
        println!("json: {json:?}");

        let mut renderer = Renderer::new_unimplemented();

        let mut obj_loader = OBJModelLoader::new(include_dir!("./tests/testmodel/obj"));
        let scene = obj_loader.load_scene("cubes", &mut renderer).unwrap();
        println!("Loaded scene: {scene:?}");

        self.renderer.create(|| renderer);
        panic!("Oh no!");
    }

    fn update(&mut self, window: &mut Window, delta_u: f64) {}

    fn draw(&mut self, window: &mut Window, delta_t: f64) {}

    fn post_draw(&mut self, window: &mut Window, delta_t: f64) {}

    fn exiting(&mut self, window: &mut Window) {}

    fn resize(&mut self, window: &mut Window, width: u32, height: u32) {}
}
