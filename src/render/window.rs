use std::iter::once;
use std::time::Instant;
use glam::Mat4;
use mvsync::block::AwaitSync;
use mvutils::utils::{TetrahedronOp, Time};
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, CommandEncoder, CommandEncoderDescriptor, IndexFormat, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, SurfaceError, TextureView, TextureViewDescriptor};
use winit::dpi::{PhysicalSize, Size};
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Fullscreen, Icon, Theme, WindowBuilder, WindowButtons, WindowId};
use crate::render::camera::{Camera2D, Camera3D};
use crate::render::common::{EffectShader, Shader, ShaderType, Texture};
use crate::render::consts::{BIND_GROUP_2D, BIND_GROUP_BATCH_3D, BIND_GROUP_EFFECT, BIND_GROUP_GEOMETRY_BATCH_3D, BIND_GROUP_GEOMETRY_MODEL_3D, BIND_GROUP_LIGHTING_3D, BIND_GROUP_MODEL_3D, BIND_GROUP_TEXTURES_2D, TEXTURE_LIMIT, VERTEX_LAYOUT_2D, VERTEX_LAYOUT_BATCH_3D, VERTEX_LAYOUT_MODEL_3D};
use crate::render::init::{State};
use crate::render::render::{EBuffer, RenderPass2D};

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
            resizeable: true,
            theme: None,
            green_eco_mode: false,
            vsync: true,
            fps: 60,
            ups: 30,
        }
    }
}

pub(crate) struct Window {
    specs: WindowSpecs,
    state: State,
    projection: Mat4,
    view: Mat4,
    render_pass_2d: RenderPass2D,
    effect_buffer: EBuffer,
    tex: Texture,
    tex2: Texture,
    frame: u64,
    pub camera_2d: Camera2D,
    pub camera_3d: Camera3D,
}

impl Window {
    /// Starts the window loop, be aware that this function only finishes when the window is closed or terminated!
    pub fn run(specs: WindowSpecs) {
        let event_loop = EventLoop::new();
        let internal_window = WindowBuilder::new()
            .with_decorations(specs.decorated)
            .with_fullscreen(specs.fullscreen.yn(Some(Fullscreen::Borderless(None)) , None))
            .with_resizable(specs.resizeable)
            .with_theme(specs.theme)
            .with_title(specs.title.as_str())
            .with_inner_size(Size::Physical(PhysicalSize::new(specs.width, specs.height)))
            .build(&event_loop).unwrap();

        let mut state = State::new(&internal_window, &specs).await_sync();

        let mut shader = Shader::new_glsl(include_str!("shaders/default.vert").to_string(), include_str!("shaders/default.frag").to_string());

        let render_pass_2d = RenderPass2D::new(
            shader.setup_pipeline(&state, VERTEX_LAYOUT_2D, &[BIND_GROUP_2D, BIND_GROUP_TEXTURES_2D]),
            &state,
            Mat4::default(),
            Mat4::default()
        );

        let mut tex = Texture::new(include_bytes!("textures/MVEngine.png").to_vec());
        tex.make(&state);
        let mut tex2 = Texture::new(include_bytes!("textures/mqxf.png").to_vec());
        tex2.make(&state);

        let effect_buffer = EBuffer::generate(&state, specs.width, specs.height);

        let mut window = Window {
            specs,
            state,
            projection: Mat4::default(),
            view: Mat4::default(),
            render_pass_2d,
            effect_buffer,
            tex,
            tex2,
            frame: 0,
            camera_2d: Default::default(),
            camera_3d: Default::default(),
        };

        let mut init_time: u128 = u128::time_nanos();
        let mut current_time: u128 = init_time;
        let time_f = 1000000000.0 / window.specs.fps as f32;
        let mut delta_f: f32 = 0.0;
        let mut frames = 0;
        let mut timer = u128::time_millis();

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::NewEvents(cause) => if cause == StartCause::Init {}
                Event::WindowEvent { event, window_id } if window_id == internal_window.id() => {
                   window.process_window_event(event, window_id, control_flow);
                }
                Event::MainEventsCleared => {
                    current_time = u128::time_nanos();
                    delta_f += (current_time - init_time) as f32 / time_f;
                    init_time = current_time;
                    if delta_f >= 1.0 {
                        internal_window.request_redraw();
                        frames += 1;
                        delta_f -= 1.0;
                        window.frame += 1;
                    }
                    if u128::time_millis() - timer > 1000 {
                        println!("{}", frames);
                        frames = 0;
                        timer += 1000;
                    }
                }
                Event::RedrawRequested(window_id) => if window_id == internal_window.id() {
                    match window.render() {
                        Ok(_) => {}
                        Err(SurfaceError::Lost) => window.resize(PhysicalSize::new(window.specs.width, window.specs.height)),
                        Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                Event::LoopDestroyed => {

                }
                _ => {}
            }
        });
    }

    fn process_window_event(&mut self, event: WindowEvent, id: WindowId, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::Resized(size) => {
                self.resize(size);
            }
            WindowEvent::ScaleFactorChanged {new_inner_size, .. } => {
                self.resize(*new_inner_size);
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

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.specs.width = size.width;
        self.specs.height = size.height;
        self.state.resize(size);
        self.effect_buffer.resize(&self.state, size.width, size.height);

        self.camera_2d.update_projection_mat(size.width as i32, size.height as i32);
        self.camera_3d.update_projection_mat(size.width as i32, size.height as i32);
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.state.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.state.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Command Encoder")
        });

        #[cfg(feature = "3d")]
        self.render_3d(&mut encoder, &view);

        self.render_2d(&mut encoder, &view);

        self.state.queue.submit(once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn render_2d(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
        let width = self.specs.width as f32;
        let height = self.specs.height as f32;
        let indices = &[0u32, 1, 2];
        let vertices = &[
            -0.5f32, -0.5, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, width, height, 0.0, 0.0, 0.0,
            0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, width, height, 0.0, 0.0, 0.0,
            0.5, -0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, width, height, 0.0, 0.0, 0.0,
        ];

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
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

        self.render_pass_2d.new_frame(&mut render_pass, self.projection, self.view);
        self.render_pass_2d.render(indices, vertices, [None; TEXTURE_LIMIT], false);

        let indices = &[0u32, 1, 2, 5, 4, 3];
        let alpha = ((self.frame as f32 / 360.0).sin() + 1.0) / 2.0;
        let vertices = &[
            -1.0f32, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, width, height, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 0.5, 1.0, 0.0, 0.0, width, height, 0.0, 0.0, 0.0,
            1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, width, height, 0.0, 0.0, 0.0,
            -1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, width, height, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 0.5, 2.0, 0.0, 0.0, width, height, 0.0, 0.0, 0.0,
            1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 2.0, 0.0, 0.0, width, height, 0.0, 0.0, 0.0,
        ];

        let mut textures = [None; TEXTURE_LIMIT];
        textures[0] = Some(&self.tex);
        textures[1] = Some(&self.tex2);

        self.render_pass_2d.render(indices, vertices, textures, false);

        self.render_pass_2d.finish();
    }

    #[cfg(feature = "3d")]
    fn render_3d(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {

    }

    pub fn create_shader(&self, vert: ShaderSource, frag: ShaderSource, usage: ShaderUsage) -> CreatedShader {
        let mut shader = Shader::new(vert.compile(ShaderType::Vertex), frag.compile(ShaderType::Fragment));
        match usage {
            ShaderUsage::Render2D => CreatedShader::Render2D(shader.setup_pipeline(&self.state, VERTEX_LAYOUT_2D, &[BIND_GROUP_2D, BIND_GROUP_TEXTURES_2D])),
            ShaderUsage::Render3D => CreatedShader::Render3D {
                batch: shader.clone().setup_pipeline(&self.state, VERTEX_LAYOUT_BATCH_3D, &[BIND_GROUP_BATCH_3D]),
                model: shader.setup_pipeline(&self.state, VERTEX_LAYOUT_MODEL_3D, &[BIND_GROUP_MODEL_3D]),
            },
            ShaderUsage::GeometryPass => CreatedShader::GeometryPass {
                batch: shader.clone().setup_pipeline(&self.state, VERTEX_LAYOUT_BATCH_3D, &[BIND_GROUP_GEOMETRY_BATCH_3D]),
                model: shader.setup_pipeline(&self.state, VERTEX_LAYOUT_MODEL_3D, &[BIND_GROUP_GEOMETRY_MODEL_3D]),
            }
        }
    }

    pub fn create_effect_shader(&self, frag: ShaderSource, usage: EffectShaderUsage) -> CreatedShader {
        let mut shader = EffectShader::new(frag.compile(ShaderType::Fragment));
        match usage {
            EffectShaderUsage::LightingPass => CreatedShader::LightingPass(shader.setup_pipeline(&self.state, &[BIND_GROUP_LIGHTING_3D])),
            EffectShaderUsage::Effect => CreatedShader::Effect(shader.setup_pipeline(&self.state, &[BIND_GROUP_EFFECT]))
        }
    }
}

pub enum ShaderSource {
    Spirv(Vec<u8>),
    Glsl(String)
}

impl ShaderSource {
    fn compile(self, shader_type: ShaderType) -> Vec<u8> {
        match self {
            ShaderSource::Spirv(v) => v,
            ShaderSource::Glsl(c) => Shader::compile_glsl(&c, shader_type)
        }
    }
}

pub enum ShaderUsage {
    Render2D,
    Render3D,
    GeometryPass
}

pub enum EffectShaderUsage {
    LightingPass,
    Effect
}

pub enum CreatedShader {
    Render2D(Shader),
    Render3D { batch: Shader, model: Shader },
    GeometryPass { batch: Shader, model: Shader },
    LightingPass(EffectShader),
    Effect(EffectShader)
}