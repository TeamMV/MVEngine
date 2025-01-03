use crate::input::raw::Input;
use crate::input::{InputAction, InputCollector, InputProcessor, KeyboardAction, MouseAction};
use crate::render::backend::Extent2D;
use crate::render::ApplicationLoopCallbacks;
use mvutils::unsafe_utils::DangerousCell;
use std::sync::Arc;
use std::time::SystemTime;
use winit::dpi::{PhysicalSize, Size};
use winit::event::{ElementState, Event, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
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

pub struct Window {
    pub(crate) info: WindowCreateInfo,

    handle: winit::window::Window,
    state: State,
    event_loop: Option<EventLoop<()>>,

    frame_time_nanos: u64,
    update_time_nanos: u64,
    delta_t: f64,
    delta_u: f64,

    input: Arc<DangerousCell<Input>>,
    input_collector: InputCollector,
}

impl Window {
    pub fn new(info: WindowCreateInfo) -> Self {
        let event_loop = EventLoop::new().unwrap();
        let window = WindowBuilder::new()
            .with_visible(false)
            .with_inner_size(Size::Physical(PhysicalSize {
                width: info.width,
                height: info.height,
            }))
            .with_title(info.title.clone())
            .with_fullscreen(info.fullscreen.then_some(Fullscreen::Borderless(None)))
            .with_resizable(info.resizable)
            .with_decorations(info.decorated)
            .with_transparent(info.transparent)
            .build(&event_loop)
            .unwrap();

        let input = Input::new();

        let input_arc = Arc::new(DangerousCell::new(input));
        let input_collector = InputCollector::new(input_arc.clone());

        Window {
            frame_time_nanos: NANOS_PER_SEC / info.fps as u64,
            update_time_nanos: NANOS_PER_SEC / info.ups as u64,
            info,
            handle: window,
            state: State::Ready,
            event_loop: Some(event_loop),
            delta_t: 0.0,
            delta_u: 0.0,
            input: input_arc,
            input_collector,
        }
    }

    pub fn run<T: ApplicationLoopCallbacks>(mut self) {
        let mut app_loop = T::new(&mut self);

        let mut time_f = SystemTime::now();
        let mut time_u = SystemTime::now();

        self.handle.set_visible(true);
        self.state = State::Running;
        self.event_loop
            .take()
            .expect("Event loop should never be None")
            .run(|event, target| match event {
                Event::AboutToWait => {
                    self.input.get_mut().loop_states();

                    let elapsed = time_u.elapsed().expect("SystemTime error").as_nanos();
                    if elapsed > self.update_time_nanos as u128 {
                        time_u = SystemTime::now();
                        self.delta_u = elapsed as f64 / NANOS_PER_SEC as f64;
                        let delta_u = self.delta_u;
                        app_loop.update(&mut self, delta_u);
                    }

                    let elapsed = time_f.elapsed().expect("SystemTime error").as_nanos();
                    if elapsed > self.frame_time_nanos as u128 {
                        time_f = SystemTime::now();
                        self.delta_t = elapsed as f64 / NANOS_PER_SEC as f64;
                        self.handle.request_redraw();
                    }
                    target.set_control_flow(ControlFlow::Poll);
                }
                Event::Suspended => {}
                Event::Resumed => {}
                Event::LoopExiting => {
                    self.state = State::Exited;
                    app_loop.exiting(&mut self);
                }
                Event::WindowEvent { window_id, event } => {
                    match event {
                        WindowEvent::ActivationTokenDone { .. } => {}
                        WindowEvent::Resized(size) => {
                            self.info.width = size.width;
                            self.info.height = size.height;
                            app_loop.resize(&mut self, size.width, size.height);
                        }
                        WindowEvent::Moved(_) => {}
                        WindowEvent::CloseRequested => target.exit(),
                        WindowEvent::Destroyed => {}
                        WindowEvent::DroppedFile(_) => {}
                        WindowEvent::HoveredFile(_) => {}
                        WindowEvent::HoveredFileCancelled => {}
                        WindowEvent::Focused(_) => {}
                        WindowEvent::KeyboardInput {
                            device_id,
                            event,
                            is_synthetic,
                        } => {
                            let input = self.input.get_mut();
                            let code = match event.physical_key {
                                PhysicalKey::Code(code) => code,
                                _ => KeyCode::Escape,
                            };

                            if let ElementState::Pressed = event.state {
                                let index = Input::key_from_winit(code);
                                if index >= 0 && index < input.keys.len() {
                                    if input.keys[index] {
                                        self.input_collector.collect(InputAction::Keyboard(
                                            KeyboardAction::Type(index),
                                        ));
                                    } else {
                                        self.input_collector.collect(InputAction::Keyboard(
                                            KeyboardAction::Type(index),
                                        ));
                                        self.input_collector.collect(InputAction::Keyboard(
                                            KeyboardAction::Press(index),
                                        ));
                                    }
                                }
                            }

                            if let ElementState::Released = event.state {
                                self.input_collector.collect(InputAction::Keyboard(
                                    KeyboardAction::Release(Input::key_from_winit(code)),
                                ));
                            }
                        }
                        WindowEvent::ModifiersChanged(_) => {}
                        WindowEvent::Ime(_) => {}
                        WindowEvent::CursorMoved {
                            device_id,
                            position,
                            ..
                        } => self
                            .input_collector
                            .collect(InputAction::Mouse(MouseAction::Move(
                                position.x as i32,
                                self.get_extent().height as i32 - position.y as i32,
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
                                self.input_collector.collect(InputAction::Mouse(
                                    MouseAction::Wheel(pos.x as f32, pos.y as f32),
                                ))
                            }
                            if let MouseScrollDelta::LineDelta(x, y) = delta {
                                self.input_collector
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
                                self.input_collector.collect(InputAction::Mouse(
                                    MouseAction::Press(Input::mouse_from_winit(button)),
                                ));
                            }

                            if let ElementState::Released = state {
                                self.input_collector.collect(InputAction::Mouse(
                                    MouseAction::Release(Input::mouse_from_winit(button)),
                                ));
                            }
                        }
                        WindowEvent::TouchpadMagnify { .. } => {}
                        WindowEvent::SmartMagnify { .. } => {}
                        WindowEvent::TouchpadRotate { .. } => {}
                        WindowEvent::TouchpadPressure { .. } => {}
                        WindowEvent::AxisMotion { .. } => {}
                        WindowEvent::Touch(_) => {}
                        WindowEvent::ScaleFactorChanged { .. } => {}
                        WindowEvent::ThemeChanged(_) => {}
                        WindowEvent::Occluded(_) => {}
                        WindowEvent::RedrawRequested => {
                            let delta_t = self.delta_t;
                            app_loop.draw(&mut self, delta_t);
                        }
                    }
                }
                _ => {}
            })
            .unwrap()
    }

    pub fn get_extent(&self) -> Extent2D {
        Extent2D {
            width: self.handle.inner_size().width,
            height: self.handle.inner_size().height,
        }
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

    pub fn get_input(&self) -> Arc<DangerousCell<Input>> {
        self.input.clone()
    }

    pub fn set_input_processor(&mut self, processor: fn(InputAction)) {
        self.input_collector.set_custom_processor(processor);
    }

    pub fn fps(&self) -> u32 {
        (1.0 / self.delta_t) as u32
    }

    pub fn ups(&self) -> u32 {
        (1.0 / self.delta_u) as u32
    }
}
