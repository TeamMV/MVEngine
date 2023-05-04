use std::iter::once;
use std::time::Instant;
use mvutils::utils::{TetrahedronOp, Time};
use wgpu::{CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, SurfaceError, TextureViewDescriptor};
use winit::dpi::{PhysicalSize, Size};
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Fullscreen, Icon, Theme, WindowBuilder, WindowButtons, WindowId};
use crate::render::init::State;

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
    pub resizeable: bool,

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
    pub vsync: bool
}

impl Default for WindowSpecs {
    fn default() -> Self {
        WindowSpecs {
            width: 800,
            height: 600,
            title: String::new(),
            fullscreen: false,
            decorated: true,
            resizeable: true,
            theme: None,
            green_eco_mode: false,
            vsync: true
        }
    }
}

pub struct Window {
    specs: WindowSpecs
}

impl Window {
    pub(crate) fn new(specs: WindowSpecs) -> Self {
        Window {
            specs
        }
    }

    /// Starts the window loop, be aware that this function only finishes when the window is closed or terminated!
    pub fn run(mut self) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_decorations(self.specs.decorated)
            .with_fullscreen(self.specs.fullscreen.yn(Some(Fullscreen::Borderless(None)) , None))
            .with_resizable(self.specs.resizeable)
            .with_theme(self.specs.theme)
            .with_title(self.specs.title.as_str())
            .with_inner_size(Size::Physical(PhysicalSize::new(self.specs.width, self.specs.height)))
            .build(&event_loop).unwrap();

        let mut state = State::new(&window, &self.specs);

        let mut init_time: u128 = u128::time_nanos();
        let mut current_time: u128 = init_time;
        let time_f = 1000000000.0 / 144.0;
        let mut delta_f: f32 = 0.0;
        let mut frames = 0;
        let mut timer = u128::time_millis();

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::NewEvents(cause) => if cause == StartCause::Init {}
                Event::WindowEvent { event, window_id } if window_id == window.id() => {
                   self.process_window_event(&mut state, event, window_id, control_flow);
                }
                Event::MainEventsCleared => {
                    current_time = u128::time_nanos();
                    delta_f += (current_time - init_time) as f32 / time_f;
                    init_time = current_time;
                    if delta_f >= 1.0 {
                        window.request_redraw();
                        frames += 1;
                    }
                    if u128::time_millis() - timer > 1000 {
                        println!("{}", frames);
                        frames = 0;
                        timer += 1000;
                    }
                }
                Event::RedrawRequested(window_id) => if window_id == window.id() {
                    match self.render(&mut state) {
                        Ok(_) => {}
                        Err(SurfaceError::Lost) => self.resize(&mut state, PhysicalSize::new(self.specs.width, self.specs.height)),
                        Err(SurfaceError::OutOfMemory) => {}//*control_flow = ControlFlow::Exit,
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                Event::LoopDestroyed => {

                }
                _ => {}
            }
        });
    }

    fn process_window_event(&mut self, state: &mut State, event: WindowEvent, id: WindowId, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::Resized(size) => {
                self.resize(state, size);
            }
            WindowEvent::ScaleFactorChanged {new_inner_size, .. } => {
                self.resize(state, *new_inner_size);
            }
            WindowEvent::Moved(pos) => {}
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }
            WindowEvent::DroppedFile(path) => {}
            WindowEvent::HoveredFile(path) => {}
            WindowEvent::HoveredFileCancelled => {}
            WindowEvent::ReceivedCharacter(c) => {}
            WindowEvent::Focused(focus) => {}
            WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {}
            WindowEvent::ModifiersChanged(mods) => {}
            WindowEvent::CursorMoved { device_id, position, .. } => {}
            WindowEvent::CursorEntered { device_id } => {}
            WindowEvent::CursorLeft { device_id } => {}
            WindowEvent::MouseWheel { device_id, delta, phase, .. } => {}
            WindowEvent::MouseInput { device_id, button, state, .. } => {}
            _ => {}
        }
    }

    fn resize(&mut self, state: &mut State, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.specs.width = size.width;
        self.specs.height = size.height;
        state.resize(size);
    }

    fn render(&mut self, state: &mut State) -> Result<(), SurfaceError> {
        let output = state.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = state.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&state.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }


        state.queue.submit(once(encoder.finish()));
        output.present();

        Ok(())
    }
}