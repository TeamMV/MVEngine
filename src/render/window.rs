use std::collections::HashMap;
use std::iter::once;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::SystemTime;

use crate::input;
use crate::input::raw::Input;
use crate::input::{InputAction, InputCollector, InputProcessor, KeyboardAction, MouseAction};
use glam::Mat4;
use mvutils::once::CreateOnce;
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use mvutils::utils::{Bytecode, Recover, TetrahedronOp};
use wgpu::core::command::RenderPass;
use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, StencilFaceState, StoreOp, SurfaceError, TextureView,
    TextureViewDescriptor,
};
use winit::dpi::{PhysicalSize, Size};
use winit::event::{ElementState, Event, MouseScrollDelta, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::keyboard::PhysicalKey;
use winit::window::{CursorIcon, Fullscreen, Theme, WindowBuilder, WindowId};

use crate::render::camera::{Camera2D, Camera3D};
use crate::render::color::RgbColor;
use crate::render::common::{EffectShader, Shader, ShaderType, Texture, TextureRegion};
#[cfg(feature = "3d")]
use crate::render::common3d::Material;
use crate::render::consts::{
    BIND_GROUP_2D, BIND_GROUP_3D, BIND_GROUP_EFFECT, BIND_GROUP_EFFECT_CUSTOM,
    BIND_GROUP_GEOMETRY_3D, BIND_GROUP_LIGHTING_3D, BIND_GROUP_MODEL_MATRIX, BIND_GROUP_TEXTURES,
    BIND_GROUP_TEXTURES_3D, FONT_SMOOTHING, MATERIAL_LIMIT, MAX_MATERIALS, VERTEX_LAYOUT_2D,
    VERTEX_LAYOUT_3D,
};
#[cfg(feature = "3d")]
use crate::render::deferred::DeferredPass;
use crate::render::draw2d::DrawContext2D;
use crate::render::init::State;
#[cfg(feature = "3d")]
use crate::render::model::ModelLoader;
use crate::render::render2d::{EBuffer, EffectPass, RenderPass2D};
#[cfg(feature = "3d")]
use crate::render::render3d::{ForwardPass, RenderPass3D};
use crate::render::text::FontLoader;
use crate::render::{consts, ApplicationLoopCallbacks};

pub struct WindowSpecs {
    /// The width of the window in pixels.
    ///
    /// Default is 800.
    pub width: u32,

    /// The height of the window in pixels.
    ///
    /// Default is 600.
    pub height: u32,

    /// The window title, which is displayed at the top of the window.
    ///
    /// Default is an empty string.
    pub title: String,

    /// Whether the window should be fullscreen.
    ///
    /// Default is false.
    pub fullscreen: bool,

    /// Whether the window should have a frame and buttons (like close, minimize and maximize)
    ///
    /// Default is true.
    pub decorated: bool,

    /// Whether the window should be resizable.
    ///
    /// Default is true.
    pub resizable: bool,

    /// Whether the window background is transparent.
    ///
    /// Default is false.
    pub transparent: bool,

    /// Dark or Light theme. None means system theme.
    ///
    /// Default is None.
    pub theme: Option<Theme>,

    /// Whether the window should reduce power consumption at the expense of worse performance by selecting an inferior GPU.
    ///
    /// Default is false.
    pub green_eco_mode: bool,

    /// Whether to sync the screen update with the time the vertical electron beam of your monitor reaches its lowest point.
    ///
    /// Default is true.
    pub vsync: bool,

    /// The maximum framerate of the window.
    ///
    /// Default is 60.
    pub fps: u32,

    /// The maximum update rate of the window.
    ///
    /// Default is 30.
    pub ups: u32,
}

impl Default for WindowSpecs {
    fn default() -> Self {
        WindowSpecs {
            width: 800,
            height: 600,
            title: String::new(),
            fullscreen: false,
            decorated: true,
            resizable: true,
            transparent: false,
            theme: None,
            green_eco_mode: false,
            vsync: true,
            fps: 60,
            ups: 30,
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum OperateState {
    Loading,
    Running,
    Paused,
}

pub struct Window<ApplicationLoop: ApplicationLoopCallbacks + 'static> {
    pub specs: DangerousCell<WindowSpecs>,
    start_time: SystemTime,

    close: DangerousCell<bool>,

    frame: DangerousCell<u64>,
    fps: DangerousCell<u64>,

    application_loop: ApplicationLoop,
    operate_state: RwLock<OperateState>,
    load_fn: fn(Arc<Window<ApplicationLoop>>),
    #[cfg(feature = "3d")]
    model_loader: CreateOnce<ModelLoader<ApplicationLoop>>,

    state: DangerousCell<State>,
    camera_2d: RwLock<Camera2D>,
    camera_3d: RwLock<Camera3D>,

    render_pass_2d: DangerousCell<RenderPass2D>,
    draw_2d: Mutex<DrawContext2D>,
    enabled_effects_2d: RwLock<Vec<String>>,

    // #[cfg(feature = "3d")]
    // deferred_pass_3d: DangerousCell<DeferredPass>,
    #[cfg(feature = "3d")]
    forward_pass_3d: DangerousCell<ForwardPass>,

    effect_shaders: RwLock<HashMap<String, Arc<EffectShader>>>,
    effect_pass: DangerousCell<EffectPass>,
    effect_buffer: DangerousCell<EBuffer>,

    input_collector: DangerousCell<InputCollector>,
    cursor: DangerousCell<Cursor>,
    prev_cursor: DangerousCell<Cursor>,
    internal_window: DangerousCell<Arc<Mutex<winit::window::Window>>>,
}

unsafe impl<T: ApplicationLoopCallbacks> Send for Window<T> {}

unsafe impl<T: ApplicationLoopCallbacks> Sync for Window<T> {}

impl<T: ApplicationLoopCallbacks + 'static> Window<T> {
    /// Starts the window loop, be aware that this function only finishes when the window is closed or terminated!
    pub fn run(mut specs: WindowSpecs, application_loop: T) {
        let event_loop = EventLoop::new().expect("Event loop already created!");
        let internal_window = WindowBuilder::new()
            .with_transparent(specs.transparent)
            .with_decorations(specs.decorated)
            .with_fullscreen(
                specs
                    .fullscreen
                    .yn(Some(Fullscreen::Borderless(None)), None),
            )
            .with_resizable(specs.resizable)
            .with_theme(specs.theme)
            .with_title(specs.title.as_str())
            .with_inner_size(Size::Physical(PhysicalSize::new(specs.width, specs.height)))
            .with_visible(false)
            .build(&event_loop)
            .unwrap();

        specs.width = internal_window.inner_size().width;
        specs.height = internal_window.inner_size().height;

        let state = State::new(&internal_window, &specs);

        let internal_window = Arc::new(Mutex::new(internal_window));

        let shader = Shader::new_glsl(
            include_str!("shaders/default.vert"),
            include_str!("shaders/default.frag"),
        );

        // #[cfg(feature = "3d")]
        // let deferred_shader = Shader::new_glsl(
        //     include_str!("shaders/deferred_geom.vert"),
        //     include_str!("shaders/deferred_geom.frag"),
        // );

        #[cfg(feature = "3d")]
        let forward_shader = Shader::new_glsl(
            include_str!("shaders/deferred_geom.vert"),
            include_str!("shaders/forward.frag"),
        );

        //TODO: separate thread render (manually called from init)
        let pixelate = EffectShader::new_glsl(include_str!("shaders/pixelate.frag"), 1)
            .setup_pipeline(&state, &[BIND_GROUP_EFFECT, BIND_GROUP_EFFECT_CUSTOM]);
        let blur = EffectShader::new_glsl(include_str!("shaders/blur.frag"), 0)
            .setup_pipeline(&state, &[BIND_GROUP_EFFECT, BIND_GROUP_EFFECT_CUSTOM]);
        let distort = EffectShader::new_glsl(include_str!("shaders/distortion.frag"), 0)
            .setup_pipeline(&state, &[BIND_GROUP_EFFECT, BIND_GROUP_EFFECT_CUSTOM]);
        let wave = EffectShader::new_glsl(include_str!("shaders/wave.frag"), 0)
            .setup_pipeline(&state, &[BIND_GROUP_EFFECT, BIND_GROUP_EFFECT_CUSTOM]);

        //TODO: separate thread render (manually called from init)
        pixelate.setup(&state, |maker| {
            maker.set_float(0, 10.0);
        });

        let mut render_pass_2d = RenderPass2D::new(
            shader.setup_pipeline(
                &state,
                VERTEX_LAYOUT_2D,
                &[BIND_GROUP_2D, BIND_GROUP_TEXTURES],
            ),
            &state,
            Mat4::default(),
            Mat4::default(),
        );

        render_pass_2d.set_smoothing(FONT_SMOOTHING);

        //#[cfg(feature = "3d")]
        //let deferred_pass_3d = DeferredPass::new(
        //    deferred_shader.setup_pipeline(
        //        &state,
        //        VERTEX_LAYOUT_3D,
        //        &[
        //            BIND_GROUP_GEOMETRY_3D,
        //            BIND_GROUP_MODEL_MATRIX,
        //            BIND_GROUP_TEXTURES_3D,
        //        ],
        //    ),
        //    &state,
        //);

        #[cfg(feature = "3d")]
        let forward_pass = ForwardPass::new(
            forward_shader.setup_pipeline(
                &state,
                VERTEX_LAYOUT_3D,
                &[
                    BIND_GROUP_GEOMETRY_3D,
                    BIND_GROUP_MODEL_MATRIX,
                    BIND_GROUP_TEXTURES_3D,
                ],
            ),
            &state,
            Mat4::default(),
            Mat4::default(),
        );

        //let mut tex = Texture::new(include_bytes!("textures/MVEngine.png").to_vec());
        //tex.make(&state);
        //let tex = TextureRegion::from(Arc::new(tex));
        //let tex = Arc::new(tex);
        //let mut tex2 = Texture::new(include_bytes!("textures/mqxf.png").to_vec());
        //tex2.make(&state);
        //let tex2 = Arc::new(tex2);

        //let t = unsafe { &crate::r::TEXTURES }.get("hello").unwrap();

        let internal_window_lock = internal_window.lock().unwrap();

        let effect_buffer = EBuffer::generate(&state, specs.width, specs.height);

        let effect_pass = EffectPass::new(&state, &effect_buffer);

        let draw_2d = DrawContext2D::new(
            Arc::new(FontLoader::new().load_default_font(&state)),
            specs.width,
            specs.height,
            internal_window_lock.scale_factor() as f32 * 96.0,
        );

        let camera_2d = Camera2D::new(specs.width, specs.height);
        let camera_3d = Camera3D::new(specs.width, specs.height);

        let window = Arc::new(Window {
            specs: specs.into(),
            application_loop,
            operate_state: OperateState::Loading.into(),
            load_fn: |_| {},
            close: false.into(),
            state: state.into(),
            start_time: SystemTime::now(),
            draw_2d: draw_2d.into(),
            render_pass_2d: render_pass_2d.into(),
            effect_buffer: effect_buffer.into(),
            effect_pass: effect_pass.into(),
            frame: 0.into(),
            fps: 0.into(),
            camera_2d: camera_2d.into(),
            camera_3d: camera_3d.into(),
            effect_shaders: HashMap::new().into(),
            enabled_effects_2d: Vec::new().into(),
            // #[cfg(feature = "3d")]
            // deferred_pass_3d: deferred_pass_3d.into(),
            #[cfg(feature = "3d")]
            model_loader: CreateOnce::new(),
            input_collector: InputCollector::new(Arc::new(RwLock::new(Input::new()))).into(),
            cursor: Cursor::Arrow.into(),
            prev_cursor: Cursor::Arrow.into(),
            internal_window: internal_window.clone().into(),
            #[cfg(feature = "3d")]
            forward_pass_3d: forward_pass.into(),
        });

        #[cfg(feature = "3d")]
        window
            .model_loader
            .create(|| ModelLoader::new(window.clone()));

        //TODO: separate thread render (manually called from init)
        window.add_effect_shader("pixelate".to_string(), CreatedShader::Effect(pixelate));
        window.add_effect_shader("blur".to_string(), CreatedShader::Effect(blur));
        window.add_effect_shader("distort".to_string(), CreatedShader::Effect(distort));
        window.add_effect_shader("wave".to_string(), CreatedShader::Effect(wave));

        let mut now = SystemTime::now();
        let mut timer = SystemTime::now();
        let time_f = 1000000000.0 / window.specs.get().fps as f32;
        let mut delta_f: f32 = 0.0;
        let mut frames = 0;

        //TODO: separate thread startup
        let clone = window.clone();
        thread::spawn(move || {
            clone.application_loop.start(clone.clone());
            *clone.operate_state.write().recover() = OperateState::Running
        });

        let id = internal_window_lock.id();

        internal_window_lock.set_visible(true);
        internal_window_lock.set_cursor_visible(true);

        drop(internal_window_lock);

        event_loop
            .run(move |event, target| match event {
                Event::NewEvents(cause) => if cause == StartCause::Init {},
                Event::WindowEvent { event, window_id } if window_id == id => {
                    if event == WindowEvent::RedrawRequested {
                        match window.render() {
                            Ok(_) => {}
                            Err(SurfaceError::Lost) => window.resize(PhysicalSize::new(
                                window.specs.get().width,
                                window.specs.get().height,
                            )),
                            Err(SurfaceError::OutOfMemory) => target.exit(),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    } else {
                        window.process_window_event(event, window_id, target);
                    }
                }
                Event::AboutToWait => {
                    if *window.close.get() {
                        target.exit();
                        return;
                    }
                    delta_f += now
                        .elapsed()
                        .unwrap_or_else(|e| {
                            panic!(
                                "System clock error: Time elapsed of -{}ns is not valid!",
                                e.duration().as_nanos()
                            )
                        })
                        .as_nanos() as f32
                        / time_f;
                    now = SystemTime::now();
                    if delta_f >= 1.0 {
                        let lock = internal_window.lock().unwrap();
                        lock.request_redraw();
                        drop(lock);
                        frames += 1;
                        delta_f -= 1.0;
                        *window.frame.get_mut() += 1;
                    }
                    if timer
                        .elapsed()
                        .unwrap_or_else(|e| {
                            panic!(
                                "System clock error: Time elapsed of -{}ms is not valid!",
                                e.duration().as_millis()
                            )
                        })
                        .as_millis()
                        >= 1000
                    {
                        *window.fps.get_mut() = frames;
                        println!("{}", frames);
                        frames = 0;
                        timer = SystemTime::now();
                    }
                }
                Event::LoopExiting => {
                    window.application_loop.exit(window.clone());
                }
                _ => {}
            })
            .expect("Error: Window main loop crashed!");
    }

    fn process_window_event(
        self: &Arc<Self>,
        event: WindowEvent,
        _id: WindowId,
        target: &EventLoopWindowTarget<()>,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                self.resize(size);
            }
            //WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer } => {}
            WindowEvent::Moved(_pos) => {}
            WindowEvent::CloseRequested => {
                target.exit();
            }
            WindowEvent::DroppedFile(_path) => {}
            WindowEvent::HoveredFile(_path) => {}
            WindowEvent::HoveredFileCancelled => {}
            WindowEvent::Focused(_focus) => {}
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                let index = if let PhysicalKey::Code(code) = event.physical_key {
                    Input::key_from_winit(code)
                } else {
                    return;
                };
                if let ElementState::Pressed = event.state {
                    let tmp = self.input();
                    let input = tmp.read().recover();
                    if index > 0 && index < input.keys.len() {}
                    if input.keys[index] {
                        self.input_collector
                            .get_mut()
                            .collect(InputAction::Keyboard(KeyboardAction::Type(index)));
                    } else {
                        self.input_collector
                            .get_mut()
                            .collect(InputAction::Keyboard(KeyboardAction::Type(index)));
                        self.input_collector
                            .get_mut()
                            .collect(InputAction::Keyboard(KeyboardAction::Press(index)));
                    }
                    drop(input);
                }

                if let ElementState::Released = event.state {
                    self.input_collector
                        .get_mut()
                        .collect(InputAction::Keyboard(KeyboardAction::Release(index)));
                }
            }
            WindowEvent::ModifiersChanged(_mods) => {}
            WindowEvent::CursorMoved {
                device_id,
                position,
                ..
            } => self
                .input_collector
                .get_mut()
                .collect(InputAction::Mouse(MouseAction::Move(
                    position.x as i32,
                    self.specs.get().height as i32 - position.y as i32,
                ))),
            WindowEvent::CursorEntered { .. } => {}
            WindowEvent::CursorLeft { .. } => {}
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
                ..
            } => {
                if let MouseScrollDelta::PixelDelta(pos) = delta {
                    self.input_collector
                        .get_mut()
                        .collect(InputAction::Mouse(MouseAction::Wheel(
                            pos.x as f32,
                            pos.y as f32,
                        )))
                }
                if let MouseScrollDelta::LineDelta(x, y) = delta {
                    self.input_collector
                        .get_mut()
                        .collect(InputAction::Mouse(MouseAction::Wheel(x, y)))
                }
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
                ..
            } => {
                if let ElementState::Pressed = state {
                    self.input_collector
                        .get_mut()
                        .collect(InputAction::Mouse(MouseAction::Press(
                            Input::mouse_from_winit(button),
                        )));
                }

                if let ElementState::Released = state {
                    self.input_collector.get_mut().collect(InputAction::Mouse(
                        MouseAction::Release(Input::mouse_from_winit(button)),
                    ));
                }
            }
            _ => {}
        }
    }

    fn resize(self: &Arc<Self>, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.specs.get_mut().width = size.width;
        self.specs.get_mut().height = size.height;
        self.state.get_mut().resize(size);
        self.effect_buffer
            .get_mut()
            .resize(self.state.get(), size.width, size.height);

        self.effect_pass
            .get_mut()
            .rebind(self.state.get(), self.effect_buffer.get());

        self.camera_2d
            .write()
            .recover()
            .update_projection(size.width, size.height);
        self.camera_3d
            .write()
            .recover()
            .update_projection(size.width, size.height);

        self.draw_2d
            .lock()
            .recover()
            .resize(size.width, size.height);
    }

    fn render(self: &Arc<Self>) -> Result<(), SurfaceError> {
        let state = *self.operate_state.read().recover();
        if state == OperateState::Paused {
            return Ok(());
        }
        let output = self.state.get().surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder =
            self.state
                .get()
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Command Encoder"),
                });

        if state == OperateState::Running {
            self.application_loop.draw(self.clone());
            self.application_loop.effect(self.clone());

            let input = self.input_collector.get_mut().get_input();
            input.write().recover().loop_states();

            #[cfg(feature = "3d")]
            self.render_3d(&mut encoder, &view);

            self.render_2d(&mut encoder, &view);
        } else {
            (self.load_fn)(self.clone());
        }

        self.state.get().queue.submit(once(encoder.finish()));

        output.present();

        Ok(())
    }

    fn render_2d(self: &Arc<Self>, encoder: &mut CommandEncoder, view: &TextureView) {
        let encoder = encoder as *mut CommandEncoder;

        macro_rules! gen_pass {
            ($e:ident, $v:expr) => {
                unsafe { $e.as_mut().unwrap() }.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: $v,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            }),
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                })
            };
        }

        unsafe { encoder.as_mut().unwrap() }.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let mut effects = self.enabled_effects_2d.write().recover();

        let current = if effects.len() > 0 {
            self.effect_pass.get_mut().new_frame(
                SystemTime::now()
                    .duration_since(self.start_time)
                    .expect("System time error")
                    .as_secs_f32(),
                self.specs.get().width,
                self.specs.get().height,
            );
            self.effect_buffer.get().get_write()
        } else {
            view
        };
        let mut render_pass = gen_pass!(encoder, current);

        let cam = self.camera_2d.read().recover();

        self.render_pass_2d.get_mut().new_frame(
            &mut render_pass,
            cam.get_projection(),
            cam.get_view(),
        );

        drop(cam);

        self.draw_2d.lock().recover().reset_transformations();

        self.draw_2d
            .lock()
            .recover()
            .render(self.render_pass_2d.get_mut());

        self.render_pass_2d.get_mut().finish();

        drop(render_pass);

        if effects.len() > 0 {
            let shaders = self.effect_shaders.read().recover();
            let mut remaining = effects.len();
            for shader in effects.drain(..) {
                self.effect_buffer.get_mut().swap();
                self.effect_pass.get_mut().swap();
                let mut pass = if remaining == 1 {
                    gen_pass!(encoder, view)
                } else {
                    gen_pass!(encoder, self.effect_buffer.get_mut().get_write())
                };
                self.effect_pass.get_mut().new_target(&mut pass);
                let effect_shader = shaders.get(shader.as_str());
                if let Some(effect_shader) = effect_shader {
                    self.effect_pass.get_mut().render(effect_shader.clone());
                } else {
                    panic!("Effect shader not found: {}", shader);
                }
                remaining -= 1;
            }
            self.effect_pass.get_mut().finish();
        }
    }

    #[cfg(feature = "3d")]
    fn render_3d(self: &Arc<Self>, encoder: &mut CommandEncoder, view: &TextureView) {
        let encoder = encoder as *mut CommandEncoder;
        let array: [Option<_>; 255] = [0; 255].map(|_| None::<T>);

        let verts: &[f32] = &[
            100f32, 0.0, -10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 50f32, -10.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 1.0, 1.0, -50f32, 0.0, -10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0,
        ];
        let inds: &[u32] = &[0, 1, 2];

        let mut render_pass =
            unsafe { encoder.as_mut().unwrap() }.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        let cam = self.camera_3d.read().recover();

        self.forward_pass_3d.get_mut().new_frame(
            &mut render_pass,
            cam.get_projection(),
            cam.get_view(),
        );

        let mut mat = Material::new();
        mat.diffuse = RgbColor::red();

        let mut mats = [0; MATERIAL_LIMIT].map(|_| None);
        mats[0] = Some(Arc::new(mat));

        self.forward_pass_3d
            .get_mut()
            .render(inds, verts, &mats, &[Mat4::default()]);
        self.forward_pass_3d.get_mut().finish();

        //self.deferred_pass_3d
        //    .get_mut()
        //    .new_frame(encoder, view, Mat4::default(), Mat4::default());
        ////self.deferred_pass_3d.get_mut().render(self.model.mesh.indices.as_slice(), self.model.data_array(), &array, false, 1);
        //self.deferred_pass_3d.get_mut().finish();
    }

    pub fn create_texture(self: &Arc<Self>, binary: Bytecode) -> Texture {
        let mut tex = Texture::new(binary);
        tex.make(self.state.get());
        tex
    }

    pub fn create_shader(
        self: &Arc<Self>,
        vert: ShaderSource,
        frag: ShaderSource,
        usage: ShaderUsage,
    ) -> CreatedShader {
        let shader = Shader::new(
            vert.compile(ShaderType::Vertex),
            frag.compile(ShaderType::Fragment),
        );
        match usage {
            ShaderUsage::Render2D => CreatedShader::Render2D(shader.setup_pipeline(
                self.state.get(),
                VERTEX_LAYOUT_2D,
                &[BIND_GROUP_2D, BIND_GROUP_TEXTURES],
            )),
            ShaderUsage::Render3D => CreatedShader::Render3D(shader.clone().setup_pipeline(
                self.state.get(),
                VERTEX_LAYOUT_3D,
                &[BIND_GROUP_3D],
            )),
            ShaderUsage::GeometryPass => CreatedShader::GeometryPass(shader.setup_pipeline(
                self.state.get(),
                VERTEX_LAYOUT_3D,
                &[BIND_GROUP_GEOMETRY_3D],
            )),
        }
    }

    pub fn create_effect_shader(
        self: &Arc<Self>,
        frag: ShaderSource,
        usage: EffectShaderUsage,
    ) -> CreatedShader {
        let size = if let EffectShaderUsage::Effect(ref size) = usage {
            *size
        } else {
            0
        };
        let shader = EffectShader::new(frag.compile(ShaderType::Fragment), size);
        match usage {
            EffectShaderUsage::LightingPass => CreatedShader::LightingPass(
                shader.setup_pipeline(self.state.get(), &[BIND_GROUP_LIGHTING_3D]),
            ),
            EffectShaderUsage::Effect(_) => CreatedShader::Effect(shader.setup_pipeline(
                self.state.get(),
                &[BIND_GROUP_EFFECT, BIND_GROUP_EFFECT_CUSTOM],
            )),
        }
    }

    pub fn add_effect_shader(self: &Arc<Self>, name: String, shader: CreatedShader) {
        if let CreatedShader::Effect(shader) = shader {
            self.effect_shaders
                .write()
                .recover()
                .insert(name, Arc::new(shader));
        } else {
            panic!(
                "Invalid shader type in shader '{}', expected Effect shader!",
                name
            );
        }
    }

    pub fn enable_effect_2d(self: &Arc<Self>, name: String) {
        self.enabled_effects_2d.write().recover().push(name);
    }

    pub fn draw_2d_pass<F: FnOnce(&mut DrawContext2D)>(self: &Arc<Self>, f: F) {
        let mut draw = self.draw_2d.lock().recover();
        f(&mut draw);
    }

    pub fn close(self: &Arc<Self>) {
        *self.close.get_mut() = true;
    }

    pub fn input(&self) -> Arc<RwLock<Input>> {
        self.input_collector.get().get_input()
    }

    pub fn set_cursor(&self, cur: Cursor) {
        if *self.cursor.get() == cur {
            return;
        }
        self.prev_cursor.replace(*self.cursor.get());
        let lock = self.internal_window.get().lock().unwrap();
        if let Cursor::None = cur {
            lock.set_cursor_visible(false);
        } else {
            lock.set_cursor_visible(true);
            lock.set_cursor_icon(cur.map_to_winit());
        }
        drop(lock);
        self.cursor.replace(cur);
    }

    pub fn get_cursor(&self) -> Cursor {
        self.cursor.get_val()
    }

    pub fn restore_cursor(&self) {
        self.set_cursor(self.prev_cursor.get_val())
    }
}

#[macro_export]
macro_rules! draw_2d {
    ($win:expr => {
        $(
            $func:ident $($param:expr),*
        );*$(;)?
    }) => {
        $win.draw_2d_pass(|draw| {
            $(
                draw.$func($($param),*);
            )*
        });
    };
}

pub enum ShaderSource {
    Spirv(Vec<u32>),
    Glsl(String),
}

impl ShaderSource {
    fn compile(self, shader_type: ShaderType) -> Vec<u32> {
        match self {
            ShaderSource::Spirv(v) => v,
            ShaderSource::Glsl(c) => Shader::compile_glsl(&c, shader_type),
        }
    }
}

pub enum ShaderUsage {
    Render2D,
    Render3D,
    GeometryPass,
}

pub enum EffectShaderUsage {
    LightingPass,
    Effect(u64),
}

pub enum CreatedShader {
    Render2D(Shader),
    Render3D(Shader),
    GeometryPass(Shader),
    LightingPass(EffectShader),
    Effect(EffectShader),
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Cursor {
    None,
    Arrow,
    Busy,
    SoftBusy,
    ResizeX,
    ResizeY,
    ResizeXY,
    Move,
    Pointer,
    Denied,
    Text,

    Crosshair,
    VerticalText,
    Cell,
    Copy,
    Grab,
    ZoomIn,
    ZoomOut,
}

impl Cursor {
    pub(crate) fn map_to_winit(&self) -> CursorIcon {
        match self {
            Cursor::None => CursorIcon::Default,
            Cursor::Arrow => CursorIcon::Default,
            Cursor::Busy => CursorIcon::Wait,
            Cursor::SoftBusy => CursorIcon::Progress,
            Cursor::ResizeX => CursorIcon::EwResize,
            Cursor::ResizeY => CursorIcon::NsResize,
            Cursor::ResizeXY => CursorIcon::NwseResize,
            Cursor::Move => CursorIcon::Move,
            Cursor::Pointer => CursorIcon::Pointer,
            Cursor::Denied => CursorIcon::NotAllowed,
            Cursor::Text => CursorIcon::Text,

            Cursor::Crosshair => CursorIcon::Crosshair,
            Cursor::VerticalText => CursorIcon::VerticalText,
            Cursor::Cell => CursorIcon::Cell,
            Cursor::Copy => CursorIcon::Copy,
            Cursor::Grab => CursorIcon::Grab,
            Cursor::ZoomIn => CursorIcon::ZoomIn,
            Cursor::ZoomOut => CursorIcon::ZoomOut,
        }
    }
}
