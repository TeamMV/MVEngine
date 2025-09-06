use std::mem;
use std::sync::atomic::Ordering;
use gl::types::GLuint;
use log::{trace, warn};
use crate::math::vec::Vec2;
use crate::rendering::control::RenderController;
use crate::rendering::shader::default::DefaultOpenGLShader;
use crate::rendering::shader::OpenGLShader;
use crate::rendering::{OpenGLRenderer, PrimitiveRenderer, RenderContext, CLEAR_FLAG};
use crate::rendering::backbuffer::{BackBuffer, BackBufferTarget};
use crate::rendering::camera::OrthographicCamera;
use crate::rendering::post::{OpenGLPostProcessRenderer, OpenGLPostProcessShader, OpenGlBlendShader, RenderTarget};
use crate::ui::rendering::WideRenderContext;
use crate::ui::styles::InheritSupplier;
use crate::window::Window;

enum Post {
    None,
    Some(PostSources)
}

struct PostSources {
    target: Option<RenderTarget>,
    shaders: Vec<OpenGLPostProcessShader>,
    index: usize
}

impl PostSources {
    fn new(window: &Window) -> Self {
        Self {
            target: None,
            shaders: vec![],
            index: 0,
        }
    }
}

pub struct RenderingPipeline<Renderer: PrimitiveRenderer> {
    renderer: Renderer,
    controller: RenderController,
    shader: OpenGLShader,
    camera: OrthographicCamera,
    post: Post,
    rendered: bool,
    dimension: (u32, u32),
    dpi: u32,
    backbuffer: BackBufferTarget,
    post_renderer: OpenGLPostProcessRenderer,
    blend_shader: Option<OpenGlBlendShader>,
}

impl<Renderer: PrimitiveRenderer> RenderingPipeline<Renderer> {
    ///constructs a new rendering pipeline using a custom renderer implementation. Note: The shader is actually created inside the method, so dont call .make() and .bind() yet!
    pub fn new(window: &Window, renderer: Renderer, mut shader: OpenGLShader) -> Result<Self, String> {
        shader.make()?;
        shader.bind()?;
        Ok(Self {
            renderer,
            controller: RenderController::new(shader.get_program_id()),
            shader,
            camera: OrthographicCamera::new(window.info().width, window.info().height),
            post: Post::None,
            rendered: false,
            dimension: (window.info().width, window.info().width),
            dpi: window.dpi(),
            backbuffer: BackBufferTarget::Screen,
            post_renderer: OpenGLPostProcessRenderer::new(window.width(), window.height()),
            blend_shader: None,
        })
    }

    pub fn create_post(&mut self, window: &Window) {
        self.post = Post::Some(PostSources::new(window))
    }

    ///Make this pipeline use a custom backbuffer. This is useful when multiple post pipelines will be combined on top of each other.
    pub fn use_custom_backbuffer(&mut self, window: &Window) {
        self.backbuffer = BackBufferTarget::Buffer(BackBuffer::new(window.width(), window.height()));
    }

    pub fn use_custom_blend_shader(&mut self, shader: OpenGlBlendShader) {
        self.blend_shader = Some(shader);
    }

    ///Note: The shader is actually created inside the method, so dont call .make() and .bind() yet!
    pub fn add_post_step(&mut self, mut shader: OpenGLPostProcessShader) {
        let r = shader.make();
        if r.is_err() {
            warn!("A post process shader could be not bound:\n{}", r.unwrap_err());
            return;
        }
        let r = shader.bind();
        if r.is_err() {
            warn!("A post process shader could be not bound:\n{}", r.unwrap_err());
            return;
        }
        if let Post::Some(sources) = &mut self.post {
            sources.shaders.push(shader);
        } else {
            warn!("Cannot add post step as this rendering pipeline is not in post-mode!")
        }
    }

    ///Recreates the renderers and everything
    pub fn resize(&mut self, window: &Window) {
        self.camera.update_projection(window.info().width, window.info().height);
        self.camera.update_view();
        self.renderer.recreate(window);
        self.post_renderer = OpenGLPostProcessRenderer::new(window.width(), window.height());
        if let BackBufferTarget::Buffer(bb) = &mut self.backbuffer {
            *bb = BackBuffer::new(window.width(), window.height());
        }
        self.dimension = (window.info().width, window.info().height);
        self.dpi = window.dpi();
    }

    pub fn begin_frame(&mut self) {
        trace!("Beginning pipeline");
        if let Post::Some(_) = self.post {
            OpenGLRenderer::enable_depth_buffer();
        }
        self.rendered = false;
        if let Post::Some(sources) = &mut self.post {
            sources.index = 0;
        }
        self.shader.use_program();
    }

    /// Advances the pipeline by executing the next post process shader and drawing to the screen when that was the last one
    pub fn advance<F: Fn(&mut OpenGLPostProcessShader)>(&mut self, window: &Window, f: F) {
        if let Post::Some(sources) = &mut self.post {
            if !self.rendered {
                let prev_clear = CLEAR_FLAG.swap(true, Ordering::Release);
                let target = self.controller.draw_to_target(window, &self.camera, &mut self.renderer, &mut self.shader);
                CLEAR_FLAG.store(prev_clear, Ordering::Release);
                self.post_renderer.set_target(target);
                self.rendered = true;
            } else {
                if sources.index >= sources.shaders.len() {
                    warn!("Illegal call to advance() on RenderingPipeline as there are no more shaders to process!");
                } else {
                    let shader = &mut sources.shaders[sources.index];
                    shader.use_program();
                    f(shader);
                    trace!("Running shader in pipeline");
                    self.post_renderer.run_shader(shader);
                    sources.index += 1;
                    //Idk i am more comfortable with >= than == and does it even matter
                    if sources.index >= sources.shaders.len() {
                        self.end_frame();
                        //trace!("Drew pipeline to screen.");
                    }
                }
            }
        } else {
            if !self.rendered {
                OpenGLRenderer::blend();
                if let BackBufferTarget::Buffer(bb) = &self.backbuffer {
                    trace!("Using backbuffer fbo: {}", bb.fbo);
                } else {
                    trace!("No fbo used!");
                }
                self.controller.draw(window, &self.camera, &mut self.renderer, &mut self.shader, &mut self.backbuffer);
                trace!("Drew pipeline to backbuffer.");
                self.rendered = true;
            } else {
                warn!("A non post-mode pipeline that has been rendered already was called advance() on! Did you forget begin_frame()?");
            }
        }
    }

    /// Skips the current post process step and draws the result to the screen if there is nothing left
    pub fn skip(&mut self) {
        if let Post::Some(sources) = &mut self.post {
            if sources.index < sources.shaders.len() {
                sources.index += 1;
                trace!("Skipped post-process step, new index is {}", sources.index);

                if sources.index >= sources.shaders.len() {
                    self.post_renderer.draw_to_backbuffer(&mut self.backbuffer);
                    trace!("Drew pipeline to screen after skipping.");
                }
            } else {
                warn!("Tried to skip post-process step, but none left to skip!");
            }
        } else {
            warn!("Tried to skip post-process step, but pipeline is not in post-mode!");
        }
    }

    /// If there are more pipelines to draw, initialize them using this method. This will make sure that the z-coords are properly handled
    pub fn next_pipeline<R: PrimitiveRenderer>(&mut self, other: &mut RenderingPipeline<R>) {
        let z = self.next_z();
        other.controller.set_z(z);
        if let BackBufferTarget::Buffer(bb) = &self.backbuffer {
            other.backbuffer = BackBufferTarget::Buffer(bb.clone());
        }
        trace!("Next pipeline");
        other.begin_frame();
        CLEAR_FLAG.store(false, Ordering::Release);
    }

    fn end_frame(&mut self) {
        if let Post::Some(_) = &self.post {
            if let Some(shader) = &self.blend_shader {
                self.post_renderer.draw_to_backbuffer_custom_blend_shader(&mut self.backbuffer, shader);
                trace!("Backbuffer draw with custom blend shader");
            } else {
                OpenGLRenderer::blend();
                self.post_renderer.draw_to_backbuffer(&mut self.backbuffer);
                trace!("Backbuffer draw");
            }
        }
    }

    pub fn flush(&mut self) {
        trace!("Flushing pipeline to screen");
        if let BackBufferTarget::Buffer(bb) = &mut self.backbuffer {
            trace!("With fbo: {}", bb.fbo);
            bb.swap(); //<----
            self.post_renderer.set_target(RenderTarget::from_backbuffer(bb));
            self.post_renderer.draw_to_screen();
        }
    }
}

impl RenderingPipeline<OpenGLRenderer> {
    ///constructs a new rendering pipeline using the default OpenGLRenderer and DefaultOpenGLShader.
    pub fn new_default_opengl(window: &Window) -> Result<Self, String> {
        let mut shader = DefaultOpenGLShader::new();
        shader.make()?;
        shader.bind()?;
        unsafe {
            Ok(Self {
                renderer: OpenGLRenderer::initialize(window),
                controller: RenderController::new(shader.get_program_id()),
                shader: shader.into_inner(),
                camera: OrthographicCamera::new(window.info().width, window.info().height),
                post: Post::None,
                rendered: false,
                dimension: (window.info().width, window.info().height),
                dpi: window.dpi(),
                backbuffer: BackBufferTarget::Screen,
                post_renderer: OpenGLPostProcessRenderer::new(window.width(), window.height()),
                blend_shader: None,
            })
        }
    }
}

impl<R: PrimitiveRenderer> RenderContext for RenderingPipeline<R> {
    fn controller(&mut self) -> &mut RenderController {
        &mut self.controller
    }

    fn next_z(&mut self) -> f32 {
        self.controller.next_z()
    }

    fn set_z(&mut self, z: f32) {
        self.controller.set_z(z);
    }
}

impl<R: PrimitiveRenderer> InheritSupplier for RenderingPipeline<R> {
    fn x(&self) -> i32 {
        0
    }

    fn y(&self) -> i32 {
        0
    }

    fn width(&self) -> i32 {
        self.dimension.0 as i32
    }

    fn height(&self) -> i32 {
        self.dimension.1 as i32
    }
}

impl<R: PrimitiveRenderer> WideRenderContext for RenderingPipeline<R> {
    fn dpi(&self) -> u32 {
        self.dpi
    }
}