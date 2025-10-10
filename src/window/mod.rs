use crate::input::consts::{Key, MouseButton};
use crate::input::{Input, KeyboardAction, MouseAction, RawInputEvent};
use crate::rendering::backend::Extent2D;
use crate::window::app::WindowCallbacks;
use hashbrown::HashSet;
use mvutils::once::CreateOnce;
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use parking_lot::RwLock;
use raw_window_handle::HasRawWindowHandle;
use std::num::NonZeroU32;
use std::ops::Try;
use std::sync::Arc;
use std::time::SystemTime;
use winit::dpi::{PhysicalSize, Size};
use winit::error::{EventLoopError, OsError};
use winit::event::{ElementState, Event, KeyEvent, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use winit::window::{Fullscreen, Theme, WindowBuilder};

pub mod app;

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
            title: "MVCore Application".to_string(),
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
    Vulkan,
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
        Error::Vulkan
    }
}

pub struct Window {
    pub(crate) info: WindowCreateInfo,

    handle: CreateOnce<winit::window::Window>,
    state: State,

    dpi: u32,

    frame_time_nanos: u64,
    update_time_nanos: u64,
    delta_t: f64,
    delta_u: f64,
    time_f: SystemTime,
    time_u: SystemTime,
    input: Input,
    pressed_keys: HashSet<Key>,
}

impl Window {
    pub fn new(info: WindowCreateInfo) -> Self {
        let frame_time_nanos = NANOS_PER_SEC / info.fps as u64;
        let update_time_nanos = NANOS_PER_SEC / info.ups as u64;

        let input = Input::new();

        Window {
            info,

            handle: CreateOnce::new(),
            state: State::Ready,

            dpi: 96,
            frame_time_nanos,
            update_time_nanos,
            delta_t: 0.0,
            delta_u: 0.0,
            time_f: SystemTime::now(),
            time_u: SystemTime::now(),
            input,
            pressed_keys: HashSet::new(),
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

        let window = builder.build(&mut event_loop).map_err(|x| Error::from(x))?;

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
                Event::WindowEvent { window_id, event } if window_id == self.handle.id() => {
                    match event {
                        WindowEvent::Resized(PhysicalSize { width, height }) => {
                            if let Some(_) = NonZeroU32::new(width)
                                && let Some(_) = NonZeroU32::new(height)
                            {
                                self.info.width = width;
                                self.info.height = height;
                                //self.surface.resize(&self.context, nzw,  nzh);
                                let mut app_loop = callbacks.write();
                                app_loop.resize(&mut self, width, height);
                            }
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
                        WindowEvent::KeyboardInput { event, .. } => {
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
                            if let Some(text) = text
                                && !text.is_empty()
                            {
                                let char = text.chars().next().unwrap();
                                let action = RawInputEvent::Keyboard(KeyboardAction::Char(char));
                                self.input
                                    .collector
                                    .dispatch_input(action, &this.input, this2);
                            }
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            let x = position.x as i32;
                            let y = position.y as i32;
                            self.input.collector.dispatch_input(
                                RawInputEvent::Mouse(MouseAction::Move(
                                    x,
                                    self.info.height as i32 - y,
                                )),
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
                        WindowEvent::MouseInput { state, button, .. } => {
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
                            #[cfg(feature = "timed")]
                            {
                                crate::debug::PROFILER.app_draw(|t| t.start());
                            }

                            self.time_f = SystemTime::now();

                            let mut app_loop = callbacks.write();
                            #[cfg(feature = "timed")]
                            {
                                crate::debug::PROFILER.render_batch(|t| t.start());
                                crate::debug::PROFILER.render_draw(|t| t.start());
                                crate::debug::PROFILER.ui_compute(|t| t.start());
                                crate::debug::PROFILER.ui_draw(|t| t.start());
                                crate::debug::PROFILER.input(|t| t.start());
                                crate::debug::PROFILER.waiting(|t| t.start());

                                crate::debug::PROFILER.render_batch(|t| t.pause());
                                crate::debug::PROFILER.render_draw(|t| t.pause());
                                crate::debug::PROFILER.ui_compute(|t| t.pause());
                                crate::debug::PROFILER.ui_draw(|t| t.pause());
                                crate::debug::PROFILER.input(|t| t.pause());
                                crate::debug::PROFILER.waiting(|t| t.pause());
                            }

                            app_loop.draw(&mut self, delta_t);
                            self.input.collector.end_frame();

                            #[cfg(feature = "timed")]
                            {
                                crate::debug::PROFILER.render_swap(|t| t.start());
                            }

                            #[cfg(feature = "timed")]
                            {
                                crate::debug::PROFILER.render_swap(|t| t.stop());
                            }

                            //self.ui.get_mut().end_frame();

                            #[cfg(feature = "timed")]
                            {
                                crate::debug::PROFILER.render_batch(|t| t.stop());
                                crate::debug::PROFILER.render_draw(|t| t.stop());
                                crate::debug::PROFILER.app_draw(|t| t.stop());
                                crate::debug::PROFILER.ecs_find(|t| t.stop());
                                crate::debug::PROFILER.ui_compute(|t| t.stop());
                                crate::debug::PROFILER.ui_draw(|t| t.stop());
                                crate::debug::PROFILER.input(|t| t.stop());
                                crate::debug::PROFILER.waiting(|t| t.stop());
                            }
                            app_loop.post_draw(&mut self, delta_t);
                            #[cfg(feature = "timed")]
                            {
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
                        #[cfg(feature = "timed")]
                        {
                            crate::debug::PROFILER.app_update(|t| t.start());
                        }
                        self.time_u = SystemTime::now();
                        self.delta_u = elapsed as f64 / NANOS_PER_SEC as f64;
                        let delta_u = self.delta_u;

                        let mut app_loop = callbacks.write();
                        app_loop.update(&mut self, delta_u);
                        #[cfg(feature = "timed")]
                        {
                            crate::debug::PROFILER.app_update(|t| t.stop());
                        }
                        app_loop.post_update(&mut self, delta_u);
                    }

                    let elapsed = self.time_f.elapsed().expect("SystemTime error").as_nanos();
                    if elapsed > self.frame_time_nanos as u128 {
                        self.delta_t = elapsed as f64 / NANOS_PER_SEC as f64;
                        self.handle.request_redraw();
                    }
                    target.set_control_flow(ControlFlow::Poll);
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
            self.handle
                .set_fullscreen(Some(Fullscreen::Borderless(monitor)));
        } else {
            // I don't think we need this cuz we are doing real not fake fullscreen
            // let (x, y) = self.cached_pos;
            // let (w, h) = self.cached_size;
            // self.info.width = w;
            // self.info.height = h;

            self.handle.set_fullscreen(None);
        }
    }
}
