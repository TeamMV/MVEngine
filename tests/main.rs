use itertools::Itertools;
use log::LevelFilter;
use mvengine::color::RgbColor;
use mvengine::math::vec::{Vec2, Vec4};
use mvengine::modify_style;
use mvengine::rendering::camera::OrthographicCamera;
use mvengine::rendering::control::RenderController;
use mvengine::rendering::light::{Light, LightOpenGLRenderer};
use mvengine::rendering::post::{OpenGLPostProcessRenderer, OpenGLPostProcessShader};
use mvengine::rendering::shader::light::LightOpenGLShader;
use mvengine::rendering::text::Font;
use mvengine::rendering::texture::Texture;
use mvengine::rendering::OpenGLRenderer;
use mvengine::ui::context::UiResources;
use mvengine::ui::elements::button::Button;
use mvengine::ui::elements::textbox::TextBox;
use mvengine::ui::elements::div::Div;
use mvengine::ui::elements::child::ToChild;
use mvengine::ui::elements::{UiElementCallbacks, UiElementStub};
use mvengine::ui::geometry::morph::Morph;
use mvengine::ui::rendering::ctx::DrawContext2D;
use mvengine::ui::rendering::{ctx, UiRenderer};
use mvengine::ui::res::MVR;
use mvengine::ui::styles::{UiStyle, UiValue};
use mvengine::window::app::WindowCallbacks;
use mvengine::window::{Error, Window, WindowCreateInfo};
use mvengine_proc_macro::{style_expr, ui};
use mvutils::once::CreateOnce;
use mvutils::state::State;
use parking_lot::RwLock;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;
use mvengine::audio::AudioEngine;
use mvengine::ui::styles::enums::{Direction, Origin, Position};
use mvengine::ui::styles::groups::SideStyle;
use mvengine::ui::styles::types::Dimension;

pub fn main() -> Result<(), Error> {
    mvlogger::init(std::io::stdout(), LevelFilter::Trace);

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
    draw_ctx: CreateOnce<DrawContext2D>,
    morph: CreateOnce<Morph>,
    state: CreateOnce<State<String>>,
    audio: AudioEngine
}

impl Application {
    fn new() -> Self {
        let audio = AudioEngine::setup().expect("Cannot start audio");
        //let test_sound1 = gen_sin_wave(440, audio.sample_rate(), 5000);
        //let test_sound2 = gen_sin_wave(220, audio.sample_rate(), 2000);
        //audio.play_sound(test_sound1);
        //audio.play_sound(test_sound2);
        
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
            draw_ctx: CreateOnce::new(),
            morph: CreateOnce::new(),
            state: CreateOnce::new(),
            audio,
        }
    }
}

impl WindowCallbacks for Application {
    fn post_init(&mut self, window: &mut Window) {
        unsafe {
            MVR::initialize();
            window.ui_mut().init(MVR.deref().deref());

            let mut shape = ctx::arc()
                .center(30, 30)
                .radius(50)
                .angle(45.0)
                .triangle_count(5)
                .create();

            let mut shape = MVR.resolve_shape(MVR.shape.round_rect).unwrap().clone();
            shape.invalidate();

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

            let post_render = OpenGLPostProcessRenderer::new(
                window.info().width as i32,
                window.info().height as i32,
            );

            let mut post_shader = OpenGLPostProcessShader::new(include_str!("gray.frag"));
            post_shader.make().unwrap();
            post_shader.bind().unwrap();

            let test_texture =
                Texture::from_bytes(include_bytes!("test.png")).expect("cannot red test texture!");
            let test_texture2 = Texture::from_bytes(include_bytes!("test2.png"))
                .expect("cannot red test2 texture!");

            let font_texture = Texture::from_bytes_sampled(include_bytes!("atlas.png"), true)
                .expect("cannot red font texture!");
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

            let mut btn_style = UiStyle::default();
            modify_style!(btn_style.text.color = UiValue::Just(RgbColor::white()));
            modify_style!(btn_style.background.color = UiValue::Just(RgbColor::red()));
            modify_style!(btn_style.text.stretch = UiValue::Just(Dimension::new(1.0, 1.0)));
            modify_style!(btn_style.text.size = UiValue::Just(20.0));
            modify_style!(btn_style.border.color = UiValue::Just(RgbColor::white()));
            modify_style!(btn_style.width = UiValue::Just(200));
            modify_style!(btn_style.height = UiValue::Just(50));
            btn_style.margin = SideStyle::all(UiValue::None.to_resolve());
            btn_style.padding = SideStyle::all(UiValue::None.to_resolve());

            let mut div_style = UiStyle::default();
            modify_style!(div_style.position = UiValue::Just(Position::Absolute));
            modify_style!(div_style.x = UiValue::Percent(1.0));
            modify_style!(div_style.y = UiValue::Just(200));
            modify_style!(div_style.direction = UiValue::Just(Direction::Vertical));
            modify_style!(div_style.origin = UiValue::Just(Origin::BottomRight));
            

            let state = State::new(String::new());

            let button = ui! {
                <Ui context={window.ui().context()}>
                    <Div style="position: absolute; x: 100%; y: 5bf; direction: vertical; origin: bottom_right; background.color: hsl(136, 0.62, 0.61);">
                        <Button style={btn_style.clone()}>{state.clone().map_identity()}</Button>
                        <TextBox style={btn_style} content={state.map_identity()}/>
                    </Div>
                </Ui>
            };

            self.state.create(|| state);

            let b = button.get();
            window.ui_mut().add_root(button);

            let ui_renderer = UiRenderer::new(window);
            let context = DrawContext2D::new(ui_renderer);
            self.draw_ctx.create(|| context);

            let rr = MVR.resolve_shape(MVR.shape.round_rect).unwrap();
            //for triangle in &rr.triangles {
            //    let pos = triangle.points.iter().map(|v| (v.pos.0, v.pos.1)).collect_array::<3>().unwrap();
            //    println!("{pos:?},");
            //}
            let r = MVR.resolve_shape(MVR.shape.rect).unwrap();
            let morph = rr.create_morph(r);
            self.morph.create(|| morph);
        }
    }

    fn update(&mut self, window: &mut Window, delta_u: f64) {}

    fn draw(&mut self, window: &mut Window, delta_t: f64) {
        window.ui_mut().compute_styles_and_draw(&mut self.draw_ctx);

        //let p = self.rot.sin().map(&(-1.0..1.0), &(0.0..1.0));
        //let mut frame = self.morph.animate_frame(1.0);
        //self.morph.debug_draw(&mut self.draw_ctx);
        //frame.set_translate(300, 400);
        //self.draw_ctx.shape(frame);

        let mx = window.input.mouse_x;
        let my = window.input.mouse_y;
        let w = mx - 100;
        let h = my - 100;

        //let mut rect = Rect::simple(100, 100, w, h);
        //rect.set_origin(rect.center());
        //let mut ad = MVR.resolve_adaptive(MVR.adaptive.void_rect).unwrap();
        //ad.draw(&mut *self.draw_ctx, &rect, AdaptiveFill::Drawable(Drawable::Texture(MVR.texture.test)), &window.ui.context());

        OpenGLRenderer::clear();
        self.draw_ctx.draw(window);

        self.rot += 0.5;
    }

    fn exiting(&mut self, window: &mut Window) {}

    fn resize(&mut self, window: &mut Window, width: u32, height: u32) {
        self.draw_ctx.resize(window);
    }
}
