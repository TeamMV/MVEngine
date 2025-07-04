use log::LevelFilter;
use mvengine::audio::decode::wav::WavDecoder;
use mvengine::audio::decode::AudioDecoder;
use mvengine::audio::source::SoundWithAttributes;
use mvengine::audio::{gen_sin_wave, AudioEngine};
use mvengine::color::RgbColor;
use mvengine::game::ecs::entity::EntityBehavior;
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
use mvengine::ui::elements::child::ToChild;
use mvengine::ui::elements::child::ToChildFromIterator;
use mvengine::ui::elements::div::Div;
use mvengine::ui::elements::text::Text;
use mvengine::ui::elements::textbox::TextBox;
use mvengine::ui::elements::UiElementStub;
use mvengine::ui::geometry::shape::{Shape, VertexStream};
use mvengine::ui::geometry::SimpleRect;
use mvengine::ui::rendering::UiRenderer;
use mvengine::ui::res::MVR;
use mvengine::window::app::WindowCallbacks;
use mvengine::window::{Error, Window, WindowCreateInfo};
use mvengine_proc_macro::resolve_resource;
use mvengine_proc_macro::{style_expr, ui};
use mvutils::once::CreateOnce;
use mvutils::state::State;
use parking_lot::RwLock;
use std::ops::Deref;
use std::sync::Arc;
use hashbrown::HashMap;
use mvengine::ui::parse::{parse_2xf64, parse_num, parse_num_abstract};

pub fn main() -> Result<(), Error> {
    mvlogger::init(std::io::stdout(), LevelFilter::Trace);

    //let s = include_str!("test.mss");
    //let mut lexer = MSSLexer::new(s);
    //loop {
    //    let token = lexer.next();
    //    if let MSSToken::EOF = token {
    //        break;
    //    }
    //    println!("{token:?}");
    //}

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
    draw_ctx: CreateOnce<UiRenderer>,
    state: CreateOnce<State<String>>,
    audio: AudioEngine,
}

impl Application {
    fn new() -> Self {
        let audio = AudioEngine::setup().expect("Cannot start audio");
        let decoder = WavDecoder;
        let sin = gen_sin_wave(440, 48000, 5000);
        let test_sound_a = decoder.decode(include_bytes!("fart.wav"));
        let wrapped_a = SoundWithAttributes::new(sin);
        wrapped_a.set_looping(true);
        wrapped_a.set_volume(1.0);
        wrapped_a.set_balance(1.0);

        //audio.play_sound(wrapped_a);
        // std::thread::sleep(Duration::from_millis(200));
        // wrapped_a.set_volume(0.3);
        // wrapped_a.set_balance(1.0);
        // wrapped_a.set_looping(false);
        // audio.play_sound(wrapped_a.full_clone());
        // std::thread::sleep(Duration::from_millis(200));
        // wrapped_a.set_volume(1.5);
        // wrapped_a.set_balance(0.0);
        // audio.play_sound(wrapped_a);
        // let test_sound1 = gen_sin_wave(440, audio.sample_rate(), 5000);
        // let wrapped_1 = SoundWithAttributes::new(test_sound1);
        // let test_sound2 = gen_sin_wave(220, audio.sample_rate(), 2000);
        // let wrapped_2 = SoundWithAttributes::new(test_sound2);
        // wrapped_2.set_volume(1.0);
        // audio.play_sound(wrapped_1);
        // audio.play_sound(wrapped_2);

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

            let state = State::new(String::new());

            let button = ui! {
                <Ui context={window.ui().context()}>
                    <Div style="position: absolute; x: 0; y: 0; width: 100%; height: 100%; background.color: @MVR.color/yellow; margin: none; padding: 1cm;">
                        <Div style="width: 100%; height: 100%; margin: none; direction: vertical;">
                            <Div style="width: 10cm; height: 10cm;">
                                <Div style="width: 50cm; height: 50cm; background.resource: texture; background.texture: @MVR.drawable/test; margin: none;"/>
                            </Div>
                            <TextBox placeholder="type here" style="width: 10cm; height: 1cm; text.align_x: start;"/>
                            <Div style="height: 10cm;">
                                <Text style="width: 6cm;">
                                    Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.
                                </Text>
                            </Div>
                            <Div style="direction: vertical; background.color: hsl(23, 1, 1);">
                                {
                                    (1..=20).map(|x| {
                                        ui! {
                                            <Ui context={window.ui().context()}>
                                                <Button style="width: 5cm; height: 1cm;">{x.to_string()}</Button>
                                            </Ui>
                                        }
                                    })
                                }
                            </Div>
                            <Div style="width: 10bc; height: 5bf;"/>
                        </Div>
                    </Div>
                </Ui>
            };

            self.state.create(|| state);

            let b = button.get();
            window.ui_mut().add_root(button);

            let ui_renderer = UiRenderer::new(window);
            self.draw_ctx.create(|| ui_renderer);
        }
    }

    fn update(&mut self, window: &mut Window, delta_u: f64) {}

    fn draw(&mut self, window: &mut Window, delta_t: f64) {
        if let Some(s) = resolve_resource!("@MVR.shape/rect1") {
            s.draw(&mut *self.draw_ctx, |v| {
                v.color = RgbColor::red().as_vec4();
            })
        }
        //window.ui_mut().compute_styles_and_draw(&mut self.draw_ctx);

        //let p = self.rot.sin().map(&(-1.0..1.0), &(0.0..1.0));
        //let mut frame = self.morph.animate_frame(1.0);
        //self.morph.debug_draw(&mut self.draw_ctx);
        //frame.set_translate(300, 400);
        //self.draw_ctx.shape(frame);

        //let mut rect = Rect::simple(100, 100, w, h);
        //rect.set_origin(rect.center());
        //let mut ad = MVR.resolve_adaptive(MVR.adaptive.void_rect).unwrap();
        //ad.draw(&mut *self.draw_ctx, &rect, AdaptiveFill::Drawable(Drawable::Texture(MVR.texture.test)), &window.ui.context());

        let mx = window.input.mouse_x as f32;
        let my = window.input.mouse_y as f32;
        let mouse_pos = Vec2::new(mx, my);

        if let Some(composite) = MVR.resolve_composite(MVR.composite.turret) {
            if let Some(bone) = composite.rig.skeleton.bones.get("left_turret") {
                let mut l = bone.write();
                l.set_aim(mouse_pos);
            }
            if let Some(bone) = composite.rig.skeleton.bones.get("right_turret") {
                let mut l = bone.write();
                l.set_aim(mouse_pos);
            }
            {
                let mut l = composite.rig.root_bone.write();
                l.rotate(0.5f32.to_radians());
            }

            let rect = SimpleRect::new(250, 250, 300, 300);
            //composite.draw(&mut *self.draw_ctx, MVR.deref().deref(), &rect);
            //composite.rig.debug_draw(&mut self.draw_ctx, &rect, window);
        }

        OpenGLRenderer::clear();
        self.draw_ctx.draw(window);

        self.rot += 0.5;
    }

    fn exiting(&mut self, window: &mut Window) {}

    fn resize(&mut self, window: &mut Window, width: u32, height: u32) {
        self.draw_ctx.resize(window);
    }
}
