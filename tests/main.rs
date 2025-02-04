use log::LevelFilter;
use mvengine::color::RgbColor;
use mvengine::input::consts::Key;
use mvengine::input::registry::RawInput;
use mvengine::math::vec::Vec2;
use mvengine::rendering::camera::OrthographicCamera;
use mvengine::rendering::control::RenderController;
use mvengine::rendering::light::{Light, LightOpenGLRenderer};
use mvengine::rendering::post::{OpenGLPostProcessRenderer, OpenGLPostProcessShader};
use mvengine::rendering::shader::light::LightOpenGLShader;
use mvengine::rendering::{InputVertex, Quad, Transform, Triangle, Vertex};
use mvengine::window::app::WindowCallbacks;
use mvengine::window::{Error, UninitializedWindow, Window, WindowCreateInfo};
use mvutils::once::CreateOnce;
use std::hash::Hash;
use mvengine::rendering::texture::Texture;

pub fn main() -> Result<(), Error> {
    mvlogger::init(std::io::stdout(), LevelFilter::Trace);
    let mut info = WindowCreateInfo::default();
    info.title = "Window demo".to_string();
    info.fps = 60;
    info.ups = 20;
    info.vsync = true;

    let window = Window::new(info);
    window.run::<Application>()
}

struct Application {
    renderer: CreateOnce<LightOpenGLRenderer>,
    camera: CreateOnce<OrthographicCamera>,
    controller: CreateOnce<RenderController>,
    shader: CreateOnce<LightOpenGLShader>,
    post_render: CreateOnce<OpenGLPostProcessRenderer>,
    invert_shader: CreateOnce<OpenGLPostProcessShader>,
    test_texture: CreateOnce<Texture>,
    test_texture2: CreateOnce<Texture>,
    test_texture3: CreateOnce<Texture>
}

impl WindowCallbacks for Application {
    fn new(window: UninitializedWindow) -> Self {
        Self {
            renderer: CreateOnce::new(),
            camera: CreateOnce::new(),
            controller: CreateOnce::new(),
            shader: CreateOnce::new(),
            post_render: CreateOnce::new(),
            invert_shader: CreateOnce::new(),
            test_texture: CreateOnce::new(),
            test_texture2: CreateOnce::new(),
            test_texture3: CreateOnce::new(),
        }
    }

    fn post_init(&mut self, window: &mut Window) {
        unsafe {
            let mut renderer = LightOpenGLRenderer::initialize(window);
            renderer.push_light(Light {
                pos: Vec2::new(250.0, 175.0),
                color: RgbColor::yellow().as_vec4(),
                intensity: 200.0,
                range: 200.0,
                falloff: 0.2,
            });

            renderer.push_light(Light {
                pos: Vec2::new(550.0, 175.0),
                color: RgbColor::green().as_vec4(),
                intensity: 2000.0,
                range: 500.0,
                falloff: 3.0,
            });

            let camera = OrthographicCamera::new(window.info().width, window.info().height);
            let mut shader = LightOpenGLShader::new();
            shader.make().unwrap();
            shader.bind().unwrap();
            shader.use_program();
            let controller = RenderController::new(shader.get_program_id());

            let post_render = OpenGLPostProcessRenderer::new(window.info().width as i32, window.info().height as i32);

            let mut post_shader = OpenGLPostProcessShader::new(include_str!("gray.frag"));
            post_shader.make().unwrap();
            post_shader.bind().unwrap();


            let test_texture = Texture::from_bytes(include_bytes!("test.png")).expect("cannot red test texture!");
            let test_texture2 = Texture::from_bytes(include_bytes!("test2.png")).expect("cannot red test2 texture!");
            let test_texture3 = Texture::from_bytes(include_bytes!("test3.png")).expect("cannot red test3 texture!");

            self.renderer.create(|| renderer);
            self.camera.create(|| camera);
            self.controller.create(|| controller);
            self.shader.create(|| shader);
            self.post_render.create(|| post_render);
            self.invert_shader.create(|| post_shader);
            self.test_texture.create(|| test_texture);
            self.test_texture2.create(|| test_texture2);
            self.test_texture3.create(|| test_texture3);
        }

        let registry = window.input.action_registry_mut();
        registry.create_action("forward");
        registry.create_action("left");
        registry.bind_action("forward", vec![RawInput::KeyPress(Key::W)]);
        registry.bind_action("left", vec![RawInput::KeyPress(Key::A)]);

        registry.create_action("save");
        registry.bind_action("save", vec![RawInput::KeyPress(Key::LControl), RawInput::KeyPress(Key::S)]);
    }

    fn update(&mut self, window: &mut Window, delta_u: f64) {

    }

    fn draw(&mut self, window: &mut Window, delta_t: f64) {
        if window.input.is_action("forward") {
            println!("forward is triggered");
        }

        if window.input.is_action("left") {
            println!("left is triggered");
        }

        if window.input.was_action("save") {
            println!("save was triggered");
        }

        let trns = Transform {
            translation: Default::default(),
            origin: Vec2::new(150.0, 150.0),
            scale: Vec2::splat(1.0),
            rotation: 0f32.to_radians(),
        };

        self.controller.push_quad(Quad {
            points: [
                InputVertex {
                    transform: trns.clone(),
                    pos: (100.0, 100.0, 60.0),
                    color: RgbColor::transparent().as_vec4(),
                    uv: (0.0, 0.0),
                    texture: self.test_texture.id,
                    has_texture: 1.0,
                },
                InputVertex {
                    transform: trns.clone(),
                    pos: (100.0, 500.0, 60.0),
                    color: RgbColor::transparent().as_vec4(),
                    uv: (1.0, 0.0),
                    texture: self.test_texture.id,
                    has_texture: 1.0,
                },
                InputVertex {
                    transform: trns.clone(),
                    pos: (500.0, 500.0, 60.0),
                    color: RgbColor::transparent().as_vec4(),
                    uv: (1.0, 1.0),
                    texture: self.test_texture.id,
                    has_texture: 1.0,
                },
                InputVertex {
                    transform: trns.clone(),
                    pos: (500.0, 100.0, 60.0),
                    color: RgbColor::transparent().as_vec4(),
                    uv: (0.0, 1.0),
                    texture: self.test_texture.id,
                    has_texture: 1.0,
                }
            ],
        });


        self.controller.push_quad(Quad {
            points: [
                InputVertex {
                    transform: trns.clone(),
                    pos: (500.0, 100.0, 200.0),
                    color: RgbColor::transparent().as_vec4(),
                    uv: (0.0, 0.0),
                    texture: self.test_texture3.id,
                    has_texture: 1.0,
                },
                InputVertex {
                    transform: trns.clone(),
                    pos: (500.0, 400.0, 200.0),
                    color: RgbColor::transparent().as_vec4(),
                    uv: (1.0, 0.0),
                    texture: self.test_texture3.id,
                    has_texture: 1.0,
                },
                InputVertex {
                    transform: trns.clone(),
                    pos: (700.0, 400.0, 200.0),
                    color: RgbColor::transparent().as_vec4(),
                    uv: (1.0, 1.0),
                    texture: self.test_texture3.id,
                    has_texture: 1.0,
                },
                InputVertex {
                    transform: trns,
                    pos: (700.0, 100.0, 200.0),
                    color: RgbColor::transparent().as_vec4(),
                    uv: (0.0, 1.0),
                    texture: self.test_texture3.id,
                    has_texture: 1.0,
                }
            ],
        });

        let target = self.controller.draw_to_target(window, &self.camera, &mut *self.renderer, &mut *self.shader);

        self.post_render.set_target(target);
        self.post_render.run_shader(&mut *self.invert_shader);
        self.post_render.draw_to_screen();
    }

    fn exiting(&mut self, window: &mut Window) {

    }

    fn resize(&mut self, window: &mut Window, width: u32, height: u32) {

    }
}