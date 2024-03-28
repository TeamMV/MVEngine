use ash::vk::AccessFlags;
use winit::dpi::{PhysicalSize, Size};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Fullscreen, Theme, WindowBuilder};
use crate::render::ApplicationLoopCallbacks;
use crate::render::backend::image::ImageLayout;
use crate::render::backend::swapchain::SwapchainError;
use crate::render::state::State;
use crate::render::render2d::Render2d;

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

    /// The maximum framerate of the window.
    ///
    /// Default is 60.
    pub fps: u32,

    /// The maximum update rate of the window.
    ///
    /// Default is 30.
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
            fps: 60,
            ups: 30,
        }
    }
}

pub struct Window {
    info: WindowCreateInfo,

    handle: winit::window::Window,
    event_loop: Option<EventLoop<()>>,
    state: State,
    render_2d: Render2d,
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

        let state = State::new(&info, &window);

        let render_2d = Render2d::new(&state, &info);

        Window {
            info,
            handle: window,
            event_loop: Some(event_loop),
            state,
            render_2d,
        }
    }

    pub fn run(mut self, app_loop: impl ApplicationLoopCallbacks) {
        self.handle.set_visible(true);
        self.event_loop.take().expect("Event loop should never be None").run(|event, target| {
            match event {
                Event::AboutToWait => {
                    self.handle.request_redraw();
                }
                Event::Suspended => {}
                Event::Resumed => {}
                Event::LoopExiting => {
                    self.state.get_device().wait_idle();
                }
                Event::WindowEvent { window_id, event } => {
                    match event {
                        WindowEvent::ActivationTokenDone { .. } => {}
                        WindowEvent::Resized(size) => {
                            self.info.width = size.width;
                            self.info.height = size.height;
                            self.state.resize(self.info.width, self.info.height);
                            self.render_2d.resize(&self.state, self.info.width, self.info.height);
                        }
                        WindowEvent::Moved(_) => {}
                        WindowEvent::CloseRequested => target.exit(),
                        WindowEvent::Destroyed => {}
                        WindowEvent::DroppedFile(_) => {}
                        WindowEvent::HoveredFile(_) => {}
                        WindowEvent::HoveredFileCancelled => {}
                        WindowEvent::Focused(_) => {}
                        WindowEvent::KeyboardInput { .. } => {}
                        WindowEvent::ModifiersChanged(_) => {}
                        WindowEvent::Ime(_) => {}
                        WindowEvent::CursorMoved { .. } => {}
                        WindowEvent::CursorEntered { .. } => {}
                        WindowEvent::CursorLeft { .. } => {}
                        WindowEvent::MouseWheel { .. } => {}
                        WindowEvent::MouseInput { .. } => {}
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
                            if self.render().is_err() {
                                self.state.resize(self.info.width, self.info.height);
                                self.render_2d.resize(&self.state, self.info.width, self.info.height);
                            }
                        }
                    }
                }
                _ => {}
            }
        }).unwrap()
    }

    pub(crate) fn render(&mut self) -> Result<(), SwapchainError> {
        let image_index = self.state.begin_frame()?;
        let cmd = self.state.get_current_command_buffer();
        let framebuffer = self.state.get_current_framebuffer();

        self.render_2d.draw(&self.state, cmd);

        //framebuffer.get_image(0).transition_layout(ImageLayout::PresentSrc, Some(cmd), AccessFlags::empty(), AccessFlags::empty());

        self.state.end_frame()
    }
}