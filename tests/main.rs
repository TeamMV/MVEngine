use log::LevelFilter;
use mvengine::audio::decode::wav::WavDecoder;
use mvengine::audio::decode::AudioDecoder;
use mvengine::audio::source::SoundWithAttributes;
use mvengine::audio::{gen_sin_wave, AudioEngine};
use mvengine::game::ecs::entity::EntityBehavior;
use mvengine::rendering::pipeline::RenderingPipeline;
use mvengine::rendering::OpenGLRenderer;
use mvengine::ui::context::UiResources;
use mvengine::ui::elements::button::Button;
use mvengine::ui::elements::child::ToChild;
use mvengine::ui::elements::child::ToChildFromIterator;
use mvengine::ui::elements::div::Div;
use mvengine::ui::elements::text::Text;
use mvengine::ui::elements::textbox::TextBox;
use mvengine::ui::elements::checkbox::CheckBox;
use mvengine::ui::elements::UiElementStub;
use mvengine::ui::geometry::shape::{shapes, VertexStream};
use mvengine::ui::res::MVR;
use mvengine::window::app::WindowCallbacks;
use mvengine::window::{Error, Window, WindowCreateInfo};
use mvengine::{debug, expect_element_by_id, modify_style};
use mvengine_proc_macro::resolve_resource;
use mvengine_proc_macro::{style_expr, ui};
use mvutils::once::CreateOnce;
use mvutils::state::State;
use parking_lot::RwLock;
use std::ops::Deref;
use std::sync::Arc;
use bytebuffer::ByteBuffer;
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::save::Savable;
use mvengine::color::RgbColor;
use mvengine::ui::attributes::UiState;
use mvengine::ui::geometry::shape::msfx::minifier::MSFXMinifier;
use mvengine::ui::styles::{UiStyle, UiStyleWriteObserver};

pub fn main() -> Result<(), Error> {
    mvlogger::init(std::io::stdout(), LevelFilter::Debug);

    // let data = include_str!("test.msfx");
    // let ast = MSFXParser::parse(data).unwrap();
    // println!("{:?}", ast);
    // let mut minifier = MSFXMinifier::new();
    // let ast = minifier.minify(ast);
    // println!("{:?}", ast);
    //
    // let mut buf = ByteBuffer::new_le();
    // ast.save(&mut buf);
    // let mut file = OpenOptions::new().create(true).truncate(true).write(true).open("compiled.msb").unwrap();
    // file.write_all(buf.as_bytes()).unwrap();
    //
    // let mut executor = MSFXExecutor::new();
    // let mut inputs = HashMap::new();
    // inputs.insert("num".to_string(), 1.0.into());
    // println!("\nOutput:");
    // let result = executor.run_debug(&ast, inputs);
    // println!();
    //
    // match result {
    //     Ok((ret, variables)) => {
    //         println!("Variables:");
    //         for (name, variable) in variables {
    //             println!("{name} = {:?}", variable);
    //         }
    //         println!("Return:");
    //         match ret {
    //             Return::Shape(s) => println!("{:?}", s),
    //             Return::Adaptive(a) => println!("{:?}", a),
    //         }
    //     }
    //     Err((err, variables)) => {
    //         println!("Variables:");
    //         for (name, variable) in variables {
    //             println!("{name} = {:?}", variable);
    //         }
    //         println!("Error:");
    //         println!("{err}");
    //     }
    // }

    // exit(0);

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
    rot: f32,
    draw_ctx: CreateOnce<RenderingPipeline<OpenGLRenderer>>,
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

            let state = State::new(String::new());

            let button = ui! {
                <Ui context={window.ui().context()}>
                    <Div style="position: absolute; x: 0; y: 0; width: 100%; height: 100%; background.color: @MVR.color/yellow; margin: none; padding: 1cm;">
                        <Div style="width: 100%; height: 100%; margin: none; direction: vertical;">
                            <Div style="width: 10cm; height: 10cm;">
                                <Div style="width: 50cm; height: 50cm; background.resource: texture; background.texture: @MVR.drawable/test; margin: none;"/>
                            </Div>
                            <TextBox placeholder="type here" style="width: 10cm; height: 1cm; text.align_x: start; text.align_y: middle;"/>
                            <CheckBox id="my_cb" style="height: 2cm; width: 20cm; text.align_x: start; text.size: 100%; text.align_y: end;">
                                dasdasdasds
                            </CheckBox>
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

            let cb = expect_element_by_id!(button, "my_cb");
            println!("style: {:?}", cb.get().style());

            self.state.create(|| state);

            let b = button.get();
            window.ui_mut().add_root(button);

            let pipeline = RenderingPipeline::new_default_opengl(window).unwrap();
            self.draw_ctx.create(|| pipeline);
        }
    }

    fn update(&mut self, window: &mut Window, delta_u: f64) {}

    fn draw(&mut self, window: &mut Window, delta_t: f64) {
        OpenGLRenderer::clear();
        self.draw_ctx.begin_frame();
        /*if let Some(s) = resolve_resource!("@MVR.shape/rect1") {
            s.draw(&mut *self.draw_ctx, |v| {
                v.color = RgbColor::red().as_vec4();
            })
        }*/
        let area = window.area().clone();
        window.ui_mut().draw_debug(&mut self.draw_ctx, &area);

        //let p = self.rot.sin().map(&(-1.0..1.0), &(0.0..1.0));
        //let mut frame = self.morph.animate_frame(1.0);
        //self.morph.debug_draw(&mut self.draw_ctx);
        //frame.set_translate(300, 400);
        //self.draw_ctx.shape(frame);

        //let mut rect = Rect::simple(100, 100, w, h);
        //rect.set_origin(rect.center());
        //let mut ad = MVR.resolve_adaptive(MVR.adaptive.void_rect).unwrap();
        //ad.draw(&mut *self.draw_ctx, &rect, AdaptiveFill::Drawable(Drawable::Texture(MVR.texture.test)), &window.ui.context());

        /*let mx = window.input.mouse_x as f32;
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
        }*/

        self.draw_ctx.advance(window, |_| {});

        self.rot += 0.5;
    }

    fn post_draw(&mut self, window: &mut Window, delta_t: f64) {
        //debug::print_summary(1000);
    }

    fn exiting(&mut self, window: &mut Window) {}

    fn resize(&mut self, window: &mut Window, width: u32, height: u32) {
        self.draw_ctx.resize(window);
    }
}
