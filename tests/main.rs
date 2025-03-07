use log::LevelFilter;
use mvengine::color::RgbColor;
use mvengine::input::consts::Key;
use mvengine::input::registry::RawInput;
use mvengine::math::vec::{Vec2, Vec4};
use mvengine::rendering::camera::OrthographicCamera;
use mvengine::rendering::control::RenderController;
use mvengine::rendering::light::{Light, LightOpenGLRenderer};
use mvengine::rendering::post::{OpenGLPostProcessRenderer, OpenGLPostProcessShader};
use mvengine::rendering::shader::light::LightOpenGLShader;
use mvengine::rendering::text::Font;
use mvengine::rendering::texture::Texture;
use mvengine::rendering::OpenGLRenderer;
use mvengine::ui::rendering::{ctx, UiRenderer};
use mvengine::window::app::WindowCallbacks;
use mvengine::window::{Error, UninitializedWindow, Window, WindowCreateInfo};
use mvutils::once::CreateOnce;
use std::hash::Hash;
use std::ops::Deref;
use mvengine::graphics::comp::parse::MRFParser;
use mvengine::graphics::comp::rig::Rig;
use mvengine::modify_style;
use mvengine::ui::context::UiContext;
use mvengine::ui::elements::button::Button;
use mvengine::ui::elements::{Element, UiElement, UiElementCallbacks, UiElementStub};
use mvengine::ui::rendering::ctx::DrawContext2D;
use mvengine::ui::styles::{UiStyle, UiValue, DEFAULT_STYLE};
use mvengine_proc_macro::ui;

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
    font: CreateOnce<Font>,
    rot: f32,
    button: CreateOnce<Element>,
    draw_ctx: CreateOnce<DrawContext2D>,
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
            font: CreateOnce::new(),
            rot: 0.0,
            button: CreateOnce::new(),
            draw_ctx: CreateOnce::new(),
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

            renderer.set_ambient(Vec4::splat(1.0));

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


            let font_texture = Texture::from_bytes_sampled(include_bytes!("atlas.png"), true).expect("cannot red font texture!");
            let font = Font::new(font_texture, include_bytes!("data.font")).unwrap();

            self.renderer.create(|| renderer);
            self.camera.create(|| camera);
            self.controller.create(|| controller);
            self.shader.create(|| shader);
            self.post_render.create(|| post_render);
            self.invert_shader.create(|| post_shader);
            self.test_texture.create(|| test_texture);
            self.test_texture2.create(|| test_texture2);
            self.font.create(|| font);

            println!("1");
            DEFAULT_STYLE.deref();
            println!("2");
            let a = DEFAULT_STYLE.created();
            println!("{a}");
            let mut style = UiStyle::default();
            modify_style!(style.text.color = UiValue::Just(RgbColor::white()));
            modify_style!(style.background.color = UiValue::Just(RgbColor::blue()));

            let button = ui! {
                <Ui context={window.ui().context()}>
                    <Button style={style}>Hello</Button>
                </Ui>
            };
            self.button.create(|| button);

            let ui_renderer = UiRenderer::new(window);
            let context = DrawContext2D::new(ui_renderer);
            self.draw_ctx.create(|| context);
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

        let trns = ctx::transform()
            .translate(400, 350)
            .rotate(self.rot)
            .get();

        let button = self.button.get_mut();
        button.compute_styles();
        button.draw(&mut self.draw_ctx);

        OpenGLRenderer::clear();
        self.controller.draw(window, &self.camera, &mut *self.renderer, &mut *self.shader);

        self.rot += 1.0;
    }

    fn exiting(&mut self, window: &mut Window) {

    }

    fn resize(&mut self, window: &mut Window, width: u32, height: u32) {

    }
}