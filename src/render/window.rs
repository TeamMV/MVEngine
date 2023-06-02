use std::cell::{Cell, UnsafeCell};
use std::collections::HashMap;
use std::iter::once;
use std::sync::{Arc, RwLock};
use std::time::{Instant, SystemTime};

use glam::Mat4;
use mvsync::block::AwaitSync;
use mvutils::once::{CreateOnce, Lazy};
use mvutils::unsafe_utils::DangerousCell;
use mvutils::utils::{Bytecode, Recover, TetrahedronOp, Time};
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, CommandEncoder, CommandEncoderDescriptor, IndexFormat, LoadOp, Maintain, MaintainBase, Operations, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, SurfaceError, TextureView, TextureViewDescriptor};
use winit::dpi::{PhysicalSize, Size};
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Fullscreen, Icon, Theme, WindowBuilder, WindowButtons, WindowId};

use crate::{ApplicationLoopCallbacks, MVCore, setup};
use crate::render::camera::{Camera2D, Camera3D};
use crate::render::color::{Color, RGB};
use crate::render::common::{EffectShader, Shader, ShaderType, Texture, TextureRegion};
use crate::render::consts::{BIND_GROUP_2D, BIND_GROUP_BATCH_3D, BIND_GROUP_EFFECT, BIND_GROUP_EFFECT_CUSTOM, BIND_GROUP_GEOMETRY_BATCH_3D, BIND_GROUP_GEOMETRY_MODEL_3D, BIND_GROUP_LIGHTING_3D, BIND_GROUP_MODEL_3D, BIND_GROUP_MODEL_MATRIX, BIND_GROUP_TEXTURES, MAX_TEXTURES, TEXTURE_LIMIT, VERTEX_LAYOUT_2D, VERTEX_LAYOUT_BATCH_3D, VERTEX_LAYOUT_MODEL_3D};
use crate::render::deferred::DeferredPass;
use crate::render::draw::Draw2D;
use crate::render::init::State;
use crate::render::model::{Model, ModelFileType, ModelLoader};
use crate::render::render::{EBuffer, EffectPass, RenderPass2D};
use crate::render::RenderCore;
#[cfg(feature = "3d")]
use crate::render::text::FontLoader;

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
            theme: None,
            green_eco_mode: false,
            vsync: true,
            fps: 60,
            ups: 30,
        }
    }
}

pub struct Window<ApplicationLoop: ApplicationLoopCallbacks + 'static> {
    specs: DangerousCell<WindowSpecs>,
    application_loop: ApplicationLoop,
    state: DangerousCell<State>,
    start_time: SystemTime,
    draw_2d: RwLock<Draw2D>,
    render_pass_2d: DangerousCell<RenderPass2D>,
    #[cfg(feature = "3d")]
    deferred_pass_3d: DangerousCell<DeferredPass>,
    effect_pass: DangerousCell<EffectPass>,
    effect_buffer: DangerousCell<EBuffer>,
    frame: DangerousCell<u64>,
    camera_2d: RwLock<Camera2D>,
    camera_3d: RwLock<Camera3D>,
    effect_shaders: RwLock<HashMap<String, Arc<EffectShader>>>,
    enabled_effects_2d: RwLock<Vec<String>>,
    model_loader: CreateOnce<ModelLoader<ApplicationLoop>>
}

unsafe impl<T: ApplicationLoopCallbacks> Send for Window<T> {}
unsafe impl<T: ApplicationLoopCallbacks> Sync for Window<T> {}

impl<T: ApplicationLoopCallbacks + 'static> Window<T> {
    /// Starts the window loop, be aware that this function only finishes when the window is closed or terminated!
    pub fn run(mut specs: WindowSpecs, core: Arc<RenderCore>, application_loop: T) {
        let event_loop = EventLoop::new();
        let internal_window = WindowBuilder::new()
            .with_decorations(specs.decorated)
            .with_fullscreen(specs.fullscreen.yn(Some(Fullscreen::Borderless(None)) , None))
            .with_resizable(specs.resizable)
            .with_theme(specs.theme)
            .with_title(specs.title.as_str())
            .with_inner_size(Size::Physical(PhysicalSize::new(specs.width, specs.height)))
            .build(&event_loop).unwrap();

        specs.width = internal_window.inner_size().width;
        specs.height = internal_window.inner_size().height;

        let state = State::new(&internal_window, &specs);

        let shader = Shader::new_glsl(include_str!("shaders/default.vert"), include_str!("shaders/default.frag"));
        let deferred_shader = Shader::new_glsl(include_str!("shaders/deferred_geom.vert"), include_str!("shaders/deferred_geom.frag"));

        let pixelate = EffectShader::new_glsl(include_str!("shaders/pixelate.frag"), 1)
            .setup_pipeline(&state, &[BIND_GROUP_EFFECT, BIND_GROUP_EFFECT_CUSTOM]);
        let blur = EffectShader::new_glsl(include_str!("shaders/blur.frag"), 0)
            .setup_pipeline(&state, &[BIND_GROUP_EFFECT, BIND_GROUP_EFFECT_CUSTOM]);
        let distort = EffectShader::new_glsl(include_str!("shaders/distortion.frag"), 0)
            .setup_pipeline(&state, &[BIND_GROUP_EFFECT, BIND_GROUP_EFFECT_CUSTOM]);
        let wave = EffectShader::new_glsl(include_str!("shaders/wave.frag"), 0)
            .setup_pipeline(&state, &[BIND_GROUP_EFFECT, BIND_GROUP_EFFECT_CUSTOM]);

        pixelate.setup(&state, |maker| {
            maker.set_float(0, 5.0);
        });

        let render_pass_2d = RenderPass2D::new(
            shader.setup_pipeline(&state, VERTEX_LAYOUT_2D, &[BIND_GROUP_2D, BIND_GROUP_TEXTURES]),
            &state,
            Mat4::default(),
            Mat4::default()
        );

        let deferred_pass_3d = DeferredPass::new(
            deferred_shader.setup_pipeline(&state, VERTEX_LAYOUT_MODEL_3D, &[BIND_GROUP_GEOMETRY_MODEL_3D, BIND_GROUP_MODEL_MATRIX, BIND_GROUP_TEXTURES]),
            &state
        );

        let mut tex = Texture::new(include_bytes!("textures/MVEngine.png").to_vec());
        tex.make(&state);
        let tex = TextureRegion::from(Arc::new(tex));
        let tex = Arc::new(tex);
        let mut tex2 = Texture::new(include_bytes!("textures/mqxf.png").to_vec());
        tex2.make(&state);
        let tex2 = Arc::new(tex2);

        //let t = unsafe { &crate::r::TEXTURES }.get("hello").unwrap();

        let effect_buffer = EBuffer::generate(&state, specs.width, specs.height);

        let effect_pass = EffectPass::new(&state, &effect_buffer);

        let draw_2d = Draw2D::new(Arc::new(FontLoader::new().load_default_font(&state)), specs.width, specs.height, internal_window.scale_factor() as f32);

        let camera_2d = Camera2D::new(specs.width, specs.height);
        let camera_3d = Camera3D::new(specs.width, specs.height);

        let window = Arc::new(Window {
            specs: specs.into(),
            application_loop,
            state: state.into(),
            start_time: SystemTime::now(),
            draw_2d: draw_2d.into(),
            render_pass_2d: render_pass_2d.into(),
            effect_buffer: effect_buffer.into(),
            effect_pass: effect_pass.into(),
            frame: 0.into(),
            camera_2d: camera_2d.into(),
            camera_3d: camera_3d.into(),
            effect_shaders: HashMap::new().into(),
            enabled_effects_2d: Vec::new().into(),
            deferred_pass_3d: deferred_pass_3d.into(),
            model_loader: CreateOnce::new(),
        });

        window.model_loader.create(|| ModelLoader::new(window.clone()));

        window.add_effect_shader("pixelate".to_string(), CreatedShader::Effect(pixelate));
        window.add_effect_shader("blur".to_string(), CreatedShader::Effect(blur));
        window.add_effect_shader("distort".to_string(), CreatedShader::Effect(distort));
        window.add_effect_shader("wave".to_string(), CreatedShader::Effect(wave));

        let mut init_time: u128 = u128::time_nanos();
        let mut current_time: u128 = init_time;
        let time_f = 1000000000.0 / window.specs.get().fps as f32;
        let mut delta_f: f32 = 0.0;
        let mut frames = 0;
        let mut timer = u128::time_millis();

        window.application_loop.start(window.clone());

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
                        *window.frame.get_mut() += 1;
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
                        Err(SurfaceError::Lost) => window.resize(PhysicalSize::new(window.specs.get().width, window.specs.get().height)),
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

    fn process_window_event(self: &Arc<Self>, event: WindowEvent, id: WindowId, control_flow: &mut ControlFlow) {
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

    fn resize(self: &Arc<Self>, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.specs.get_mut().width = size.width;
        self.specs.get_mut().height = size.height;
        self.state.get_mut().resize(size);
        self.effect_buffer.get_mut().resize(&self.state.get(), size.width, size.height);

        self.effect_pass.get_mut().rebind(&self.state.get(), self.effect_buffer.get());

        self.camera_2d.write().recover().update_projection(size.width, size.height);
        self.camera_3d.write().recover().update_projection(size.width, size.height);

        self.draw_2d.write().recover().resize(size.width, size.height);
    }

    fn render(self: &Arc<Self>) -> Result<(), SurfaceError> {
        let output = self.state.get().surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.state.get().device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Command Encoder")
        });

        self.application_loop.draw(self.clone());

        #[cfg(feature = "3d")]
        self.render_3d(&mut encoder, &view);

        //self.enable_effect_2d("blur".to_string());
        //self.enable_effect_2d("pixelate".to_string());
        //self.enable_effect_2d("distort".to_string());
        //self.enable_effect_2d("wave".to_string());

        self.render_2d(&mut encoder, &view);

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
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
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
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        let mut effects = self.enabled_effects_2d.write().recover();

        let current = if effects.len() > 0 {
            self.effect_pass.get_mut().new_frame(SystemTime::now().duration_since(self.start_time).expect("System time error").as_secs_f32(), self.specs.get().width, self.specs.get().height);
            self.effect_buffer.get().get_write()
        } else { view };
        let mut render_pass = gen_pass!(encoder, current);

        let cam = self.camera_2d.read().recover();

        self.render_pass_2d.get_mut().new_frame(&mut render_pass, cam.get_projection(), cam.get_view());

        drop(cam);

        self.draw_2d.write().recover().reset_canvas();

        self.draw_2d.write().recover().render(self.render_pass_2d.get_mut());

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
                }
                else {
                    panic!("Effect shader not found: {}", shader);
                }
                remaining -= 1;
            }
            self.effect_pass.get_mut().finish();
        }
    }

    #[cfg(feature = "3d")]
    fn render_3d(self: &Arc<Self>, encoder: &mut CommandEncoder, view: &TextureView) {
        //let array: [Option<_>; 255] = [0; 255].map(|_| None::<T>);

        self.deferred_pass_3d.get_mut().new_frame(encoder, view, Mat4::default(), Mat4::default());
        //self.render_pass_3d_def.render(self.model.mesh.indices.as_slice(), self.model.data_array(), &array, false, 1);
    }

    pub fn create_texture(self: &Arc<Self>, binary: Bytecode) -> Texture {
        let mut tex = Texture::new(binary);
        tex.make(&self.state.get());
        tex
    }

    pub fn create_shader(self: &Arc<Self>, vert: ShaderSource, frag: ShaderSource, usage: ShaderUsage) -> CreatedShader {
        let shader = Shader::new(vert.compile(ShaderType::Vertex), frag.compile(ShaderType::Fragment));
        match usage {
            ShaderUsage::Render2D => CreatedShader::Render2D(shader.setup_pipeline(&self.state.get(), VERTEX_LAYOUT_2D, &[BIND_GROUP_2D, BIND_GROUP_TEXTURES])),
            ShaderUsage::Render3D => CreatedShader::Render3D {
                batch: shader.clone().setup_pipeline(&self.state.get(), VERTEX_LAYOUT_BATCH_3D, &[BIND_GROUP_BATCH_3D]),
                model: shader.setup_pipeline(&self.state.get(), VERTEX_LAYOUT_MODEL_3D, &[BIND_GROUP_MODEL_3D]),
            },
            ShaderUsage::GeometryPass => CreatedShader::GeometryPass {
                batch: shader.clone().setup_pipeline(&self.state.get(), VERTEX_LAYOUT_BATCH_3D, &[BIND_GROUP_GEOMETRY_BATCH_3D]),
                model: shader.setup_pipeline(&self.state.get(), VERTEX_LAYOUT_MODEL_3D, &[BIND_GROUP_GEOMETRY_MODEL_3D]),
            }
        }
    }

    pub fn create_effect_shader(self: &Arc<Self>, frag: ShaderSource, usage: EffectShaderUsage) -> CreatedShader {
        let size = if let EffectShaderUsage::Effect(ref size) = usage { *size } else { 0 };
        let shader = EffectShader::new(frag.compile(ShaderType::Fragment), size);
        match usage {
            EffectShaderUsage::LightingPass => CreatedShader::LightingPass(shader.setup_pipeline(&self.state.get(), &[BIND_GROUP_LIGHTING_3D])),
            EffectShaderUsage::Effect(_) => CreatedShader::Effect(shader.setup_pipeline(&self.state.get(), &[BIND_GROUP_EFFECT, BIND_GROUP_EFFECT_CUSTOM]))
        }
    }

    pub fn add_effect_shader(self: &Arc<Self>, name: String, shader: CreatedShader) {
        if let CreatedShader::Effect(shader) = shader {
            self.effect_shaders.write().recover().insert(name, Arc::new(shader));
        }
        else {
            panic!("Invalid shader type in shader '{}', expected Effect shader!", name);
        }
    }

    pub fn enable_effect_2d(self: &Arc<Self>, name: String) {
        self.enabled_effects_2d.write().recover().push(name);
    }

    pub fn do_draw_2d_procedure<F: FnOnce(&mut Draw2D)>(self: &Arc<Self>, f: F) {
        let mut draw = self.draw_2d.write().recover();
        f(&mut *draw);
    }

    fn a(self: &Arc<Self>) {
        use crate::draw_2d;
        let tilt = 0.5;
        draw_2d!(self => {
            reset_canvas;
            reset_color;
            use_camera true;
            chroma_tilt tilt;
            canvas 0, 0, tilt as u32 + 100, (200 * 534) / 358;
        });

        self.do_draw_2d_procedure(|draw: &mut Draw2D| {
            draw.reset_canvas();
            draw.reset_color();
            draw.use_camera(true);
            draw.chroma_tilt(tilt);
            draw.canvas(0, 0, (tilt as u32 + 100), ((200 * 534) / 358));
        });
    }
}

#[macro_export]
macro_rules! draw_2d {
    ($win:expr => {
        $(
            $func:ident$($($param:expr),*;)?
        )*
    }) => {
        $win.do_draw_2d_procedure(|draw| {
            $(
                draw.$func($($($param),*)?);
            )*
        });
    };
}

pub enum ShaderSource {
    Spirv(Vec<u32>),
    Glsl(String)
}

impl ShaderSource {
    fn compile(self, shader_type: ShaderType) -> Vec<u32> {
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
    Effect(u64)
}

pub enum CreatedShader {
    Render2D(Shader),
    Render3D { batch: Shader, model: Shader },
    GeometryPass { batch: Shader, model: Shader },
    LightingPass(EffectShader),
    Effect(EffectShader)
}