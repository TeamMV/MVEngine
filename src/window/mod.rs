pub mod app;

use std::ffi::{CStr, CString};
use std::num::{NonZeroI32, NonZeroU32};
use crate::input::consts::{Key, MouseButton};
use crate::input::{Input, KeyboardAction, MouseAction, RawInputEvent};
use crate::ui::Ui;
use crate::ui::geometry::SimpleRect;
use crate::ui::styles::InheritSupplier;
use crate::window::app::WindowCallbacks;
use hashbrown::HashSet;
use mvutils::once::CreateOnce;
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::SystemTime;
use gl::types::GLsizei;
use glutin::config::{Api, Config, ConfigTemplateBuilder, GlConfig};
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentGlContext, Version};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, SwapInterval};
use glutin_winit::{ApiPreference, DisplayBuilder, GlWindow};
use log::warn;
use raw_window_handle::HasRawWindowHandle;
use winit::dpi::{PhysicalSize, Size};
use winit::error::{EventLoopError, OsError};
use winit::event::{ElementState, Event, KeyEvent, MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoopBuilder;
use winit::window::{Fullscreen, Theme, WindowBuilder};

const NANOS_PER_SEC: u64 = 1_000_000_000;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct WindowCreateInfo {
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

    /// Whether to sync the screen update with the time the vertical electron beam of your monitor reaches its lowest point.
    ///
    /// Default is true.
    pub vsync: bool,

    /// The maximum frames in flight of the rendering API.
    ///
    /// Default is 2.
    pub max_frames_in_flight: u32,

    /// The maximum frames per second of the window.
    ///
    /// Default is 60.
    pub fps: u32,

    /// The maximum updates per second of the window.
    ///
    /// Default is 20.
    pub ups: u32,
}

impl Default for WindowCreateInfo {
    fn default() -> Self {
        WindowCreateInfo {
            width: 800,
            height: 600,
            title: "MVEngine Application".to_string(),
            fullscreen: false,
            decorated: true,
            resizable: true,
            transparent: false,
            theme: None,
            vsync: false,
            max_frames_in_flight: 2,
            fps: 60,
            ups: 20,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum State {
    Ready,
    Running,
    Exited,
}

#[derive(Debug)]
pub enum Error {
    Window(OsError),
    EventLoop(EventLoopError),
    OpenGL,
}

impl From<OsError> for Error {
    fn from(residual: OsError) -> Self {
        Error::Window(residual)
    }
}

impl From<EventLoopError> for Error {
    fn from(residual: EventLoopError) -> Self {
        Error::EventLoop(residual)
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(_: Box<dyn std::error::Error>) -> Self {
        Error::OpenGL
    }
}

fn gl_config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    configs
        .reduce(|accum, config| {
            let transparency_check = config.supports_transparency().unwrap_or(false)
                & !accum.supports_transparency().unwrap_or(false);

            if transparency_check || config.num_samples() > accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .unwrap()
}

pub struct Window {
    pub(crate) info: WindowCreateInfo,

    handle: CreateOnce<winit::window::Window>,
    context: CreateOnce<glutin::context::PossiblyCurrentContext>,
    surface: CreateOnce<glutin::surface::Surface<glutin::surface::WindowSurface>>,
    state: State,

    dpi: u32,

    frame_time_nanos: u64,
    update_time_nanos: u64,
    delta_t: f64,
    delta_u: f64,
    time_f: SystemTime,
    time_u: SystemTime,

    // cached_pos: (i32, i32),
    // cached_size: (u32, u32),

    pub input: Input,
    pressed_keys: HashSet<Key>,

    pub(crate) ui: Arc<DangerousCell<Ui>>,
}

impl Window {
    pub fn new(info: WindowCreateInfo) -> Self {
        let frame_time_nanos = NANOS_PER_SEC / info.fps as u64;
        let update_time_nanos = NANOS_PER_SEC / info.ups as u64;

        let ui = Ui::new();
        let ui = Arc::new(DangerousCell::new(ui));

        Window {
            info,

            handle: CreateOnce::new(),
            context: CreateOnce::new(),
            surface: CreateOnce::new(),
            state: State::Ready,

            dpi: 96,
            frame_time_nanos,
            update_time_nanos,
            delta_t: 0.0,
            delta_u: 0.0,
            time_f: SystemTime::now(),
            time_u: SystemTime::now(),
            input: Input::new(ui.clone()),
            pressed_keys: HashSet::new(),
            ui,
        }
    }

    pub fn run<T: WindowCallbacks + 'static>(
        mut self,
        callbacks: Arc<RwLock<T>>,
    ) -> Result<(), Error> {
        let mut event_loop = EventLoopBuilder::new().build()?;
        let builder = WindowBuilder::new()
             .with_visible(true)
            .with_title(self.info.title.clone())
            .with_decorations(self.info.decorated)
            .with_theme(self.info.theme)
            .with_resizable(self.info.resizable)
            .with_transparent(self.info.transparent)
            .with_inner_size(Size::Physical(PhysicalSize {
                width: self.info.width,
                height: self.info.height,
            }));

        let template = ConfigTemplateBuilder::new()
            .with_api(Api::OPENGL)
            .with_alpha_size(8)
            .with_single_buffering(false)
            .with_transparency(true);

        let builder = DisplayBuilder::new()
            .with_preference(ApiPreference::FallbackEgl)
            .with_window_builder(Some(builder));

        let (window, gl_config) = builder.build(&event_loop, template, gl_config_picker)?;

        let Some(mut window) = window else {
            return Err(Error::OpenGL);
        };

        let raw_window_handle = window.raw_window_handle();

        let gl_display = gl_config.display();

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(4, 6))))
            .with_profile(glutin::context::GlProfile::Compatibility)
            .build(Some(raw_window_handle));

        let mut not_current_gl_context = Some(unsafe {
            gl_display.create_context(&gl_config, &context_attributes).map_err(|_| Error::OpenGL)?
        });

        let attrs = window.build_surface_attributes(<_>::default());
        let gl_surface = unsafe {
            gl_config.display().create_window_surface(&gl_config, &attrs).unwrap()
        };

        let gl_context =
            not_current_gl_context.take().unwrap().make_current(&gl_surface).unwrap();

        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        });

        if self.info.vsync {
            gl_surface.set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap())).map_err(|_| Error::OpenGL)?;
        } else {
            gl_surface.set_swap_interval(&gl_context, SwapInterval::DontWait).map_err(|_| Error::OpenGL)?;
        }

        gl_surface.resize(&gl_context, NonZeroU32::new(self.info.width).unwrap(),  NonZeroU32::new(self.info.height).unwrap());

        // unsafe {
        //     if cfg!(windows) {
        //         let dpi_awareness_context =
        //             winapi::um::winuser::GetWindowDpiAwarenessContext(w.get_hwnd() as *mut _);
        //         self.dpi =
        //             winapi::um::winuser::GetDpiFromDpiAwarenessContext(dpi_awareness_context);
        //     } else {
        //         panic!("Illegal operating system (go buy a copy of windows for 1000$)")
        //     }
        // }

        // unsafe {
        //     bindless::load_bindless_texture_functions(&w);
        // }

        self.handle.create(|| window);
        self.context.create(|| gl_context);
        self.surface.create(|| gl_surface);

        let mut lock = callbacks.write();
        lock.post_init(&mut self);
        drop(lock);

        // self.handle.set_visible(true);
        self.state = State::Running;

        let mut error = None;

        let this = unsafe { Unsafe::cast_lifetime_mut(&mut self) };
        let this2 = unsafe { Unsafe::cast_lifetime_mut(&mut self) };

        event_loop.run(|event, target| {
            match event {
                Event::Resumed => {}
                Event::WindowEvent {
                    window_id, event
                } if window_id == self.handle.id() => {
                    match event {
                        WindowEvent::Resized(PhysicalSize { width, height }) => {
                            self.info.width = width;
                            self.info.height = height;
                            self.surface.resize(&self.context, NonZeroU32::new(width).unwrap(),  NonZeroU32::new(height).unwrap());
                            let mut app_loop = callbacks.write();
                            app_loop.resize(&mut self, width, height);
                            self.ui.get_mut().invalidate();
                        }
                        WindowEvent::Moved(_) => {}
                        WindowEvent::DroppedFile(_) => {}
                        // WindowEvent::ReceivedCharacter(ch) => {
                        //     if !ch.is_control() {
                        //         let action = RawInputEvent::Keyboard(KeyboardAction::Char(ch));
                        //         this.input
                        //             .collector
                        //             .dispatch_input(action, &self.input, this3);
                        //     }
                        // }
                        WindowEvent::Focused(_) => {}
                        WindowEvent::KeyboardInput {
                            event,
                            ..
                        } => {
                            let KeyEvent {
                                physical_key,
                                state,
                                repeat,
                                text,
                                ..
                            } = event;
                            if let Ok(code) = Key::try_from(physical_key) {
                                let event = match state {
                                    ElementState::Pressed => {
                                        if !self.pressed_keys.contains(&code) {
                                            self.pressed_keys.insert(code.clone());
                                            RawInputEvent::Keyboard(KeyboardAction::Press(code))
                                        } else {
                                            RawInputEvent::Keyboard(KeyboardAction::Type(code))
                                        }
                                    }
                                    ElementState::Released => {
                                        self.pressed_keys.remove(&code);
                                        RawInputEvent::Keyboard(KeyboardAction::Release(code))
                                    }
                                };
                                self.input
                                    .collector
                                    .dispatch_input(event, &this.input, this2);
                            }
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            let x = position.x as i32;
                            let y = position.y as i32;
                            self.input.collector.dispatch_input(
                                RawInputEvent::Mouse(MouseAction::Move(x, self.info.height as i32 - y)),
                                &this.input,
                                this2,
                            );
                            self.input.mouse_x = x;
                            self.input.mouse_y = self.info.height as i32 - y;
                        }
                        WindowEvent::MouseWheel { delta, .. } => match delta {
                            MouseScrollDelta::LineDelta(dx, dy) => {
                                self.input.collector.dispatch_input(
                                    RawInputEvent::Mouse(MouseAction::Wheel(dx, dy)),
                                    &this.input,
                                    this2,
                                );
                            }
                            MouseScrollDelta::PixelDelta(pos) => {
                                let dx = pos.x as f32;
                                let dy = pos.y as f32;
                                self.input.collector.dispatch_input(
                                    RawInputEvent::Mouse(MouseAction::Wheel(dx, dy)),
                                    &this.input,
                                    this2,
                                );
                            }
                        },
                        WindowEvent::MouseInput { state, button,  .. } => {
                            let button = MouseButton::from(button);
                            match state {
                                ElementState::Pressed => {
                                    self.input.collector.dispatch_input(
                                        RawInputEvent::Mouse(MouseAction::Press(button)),
                                        &this.input,
                                        this2,
                                    );
                                }
                                ElementState::Released => {
                                    self.input.collector.dispatch_input(
                                        RawInputEvent::Mouse(MouseAction::Release(button)),
                                        &this.input,
                                        this2,
                                    );
                                }
                            }
                        }
                        WindowEvent::TouchpadPressure { .. } => {}
                        WindowEvent::RedrawRequested => {
                            let delta_t = self.delta_t;
                            #[cfg(feature = "timed")] {
                                crate::debug::PROFILER.app_draw(|t| t.start());
                            }

                            self.time_f = SystemTime::now();

                            let mut app_loop = callbacks.write();
                            #[cfg(feature = "timed")] {
                                crate::debug::PROFILER.render_batch(|t| t.start());
                                crate::debug::PROFILER.render_draw(|t| t.start());
                                crate::debug::PROFILER.ui_compute(|t| t.start());
                                crate::debug::PROFILER.ui_draw(|t| t.start());
                                crate::debug::PROFILER.input(|t| t.start());

                                crate::debug::PROFILER.render_batch(|t| t.pause());
                                crate::debug::PROFILER.render_draw(|t| t.pause());
                                crate::debug::PROFILER.ui_compute(|t| t.pause());
                                crate::debug::PROFILER.ui_draw(|t| t.pause());
                                crate::debug::PROFILER.input(|t| t.pause());
                            }

                            app_loop.draw(&mut self, delta_t);
                            self.input.collector.end_frame();

                            #[cfg(feature = "timed")] {
                                crate::debug::PROFILER.render_swap(|t| t.start());
                            }

                            if self.surface.swap_buffers(&self.context).is_err() {
                                error = Some(Error::OpenGL);
                                target.exit();
                            };
                            #[cfg(feature = "timed")] {
                                crate::debug::PROFILER.render_swap(|t| t.stop());
                            }

                            self.ui.get_mut().end_frame();

                            #[cfg(feature = "timed")] {
                                crate::debug::PROFILER.render_batch(|t| t.stop());
                                crate::debug::PROFILER.render_draw(|t| t.stop());
                                crate::debug::PROFILER.app_draw(|t| t.stop());
                                crate::debug::PROFILER.ecs_find(|t| t.stop());
                                crate::debug::PROFILER.ui_compute(|t| t.stop());
                                crate::debug::PROFILER.ui_draw(|t| t.stop());
                                crate::debug::PROFILER.input(|t| t.stop());
                            }
                            app_loop.post_draw(&mut self, delta_t);
                            #[cfg(feature = "timed")] {
                                crate::debug::PROFILER.ecs_find(|t| t.start());
                                crate::debug::PROFILER.ecs_find(|t| t.pause());
                            }
                        }
                        WindowEvent::Touch(_) => {}
                        WindowEvent::CloseRequested => {
                            target.exit();
                        }
                        _ => {}
                    }
                }
                Event::AboutToWait => {
                    let elapsed = self.time_u.elapsed().expect("SystemTime error").as_nanos();

                    if elapsed > self.update_time_nanos as u128 {
                        #[cfg(feature = "timed")] {
                            crate::debug::PROFILER.app_update(|t| t.start());
                        }
                        self.time_u = SystemTime::now();
                        self.delta_u = elapsed as f64 / NANOS_PER_SEC as f64;
                        let delta_u = self.delta_u;

                        let mut app_loop = callbacks.write();
                        app_loop.update(&mut self, delta_u);
                        #[cfg(feature = "timed")] {
                            crate::debug::PROFILER.app_update(|t| t.stop());
                        }
                        app_loop.post_update(&mut self, delta_u);
                    }

                    let elapsed = self.time_f.elapsed().expect("SystemTime error").as_nanos();
                    if elapsed > self.frame_time_nanos as u128 {
                        self.delta_t = elapsed as f64 / NANOS_PER_SEC as f64;
                        self.handle.request_redraw();
                    }
                }
                Event::LoopExiting => {}
                Event::MemoryWarning => {}
                Event::Suspended => {}
                Event::NewEvents(_) => {}
                Event::DeviceEvent { .. } => {}
                Event::UserEvent(_) => {}
                _ => {}
            }
        })?;

        if let Some(error) = error {
            return Err(error);
        }

        self.state = State::Exited;
        let mut app_loop = callbacks.write();
        app_loop.exiting(&mut self);

        Ok(())
    }

    pub fn info(&self) -> &WindowCreateInfo {
        &self.info
    }

    pub fn dpi(&self) -> u32 {
        self.dpi
    }

    pub fn get_state(&self) -> State {
        self.state
    }

    pub fn get_handle(&self) -> &winit::window::Window {
        &self.handle
    }

    pub fn get_surface(&self) -> &glutin::surface::Surface<glutin::surface::WindowSurface> {
        &self.surface
    }

    pub fn get_context(&self) -> &glutin::context::PossiblyCurrentContext {
        &self.context
    }

    pub fn get_delta_t(&self) -> f64 {
        self.delta_t
    }

    pub fn get_delta_u(&self) -> f64 {
        self.delta_u
    }

    pub fn set_fps(&mut self, fps: u32) {
        self.info.fps = fps;
        self.frame_time_nanos = NANOS_PER_SEC / fps as u64;
    }

    pub fn set_ups(&mut self, ups: u32) {
        self.info.ups = ups;
        self.update_time_nanos = NANOS_PER_SEC / ups as u64;
    }

    pub fn fps(&self) -> u32 {
        (1.0 / self.delta_t) as u32
    }

    pub fn ups(&self) -> u32 {
        (1.0 / self.delta_u) as u32
    }

    pub fn center(&self) -> (i32, i32) {
        ((self.info.width / 2) as i32, (self.info.height / 2) as i32)
    }

    pub fn is_fullscreen(&self) -> bool {
        self.info.fullscreen
    }

    pub fn toggle_fullscreen(&mut self) {
        self.fullscreen(!self.is_fullscreen());
    }

    pub fn fullscreen(&mut self, fullscreen: bool) {
        if self.info.fullscreen == fullscreen {
            return;
        }
        self.info.fullscreen = fullscreen;
        if fullscreen {
            // self.cached_size = (self.info.width, self.info.height);
            // self.cached_pos = self.handle.inner_position().map(|p| (p.x, p.y)).unwrap_or((0, 0));

            let monitor = self.handle.current_monitor();
            self.handle.set_fullscreen(Some(Fullscreen::Borderless(monitor)));
        } else {
            // I don't think we need this cuz we are doing real not fake fullscreen
            // let (x, y) = self.cached_pos;
            // let (w, h) = self.cached_size;
            // self.info.width = w;
            // self.info.height = h;

            self.handle.set_fullscreen(None);
        }
    }

    pub fn ui(&self) -> &Ui {
        self.ui.get()
    }

    pub fn ui_mut(&mut self) -> &mut Ui {
        self.ui.get_mut()
    }

    pub fn area(&self) -> SimpleRect {
        SimpleRect::new(0, 0, self.width(), self.height())
    }
}
