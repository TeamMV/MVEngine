use log::{trace, warn};
use crate::math::vec::Vec2;
use crate::rendering::control::RenderController;
use crate::rendering::shader::default::DefaultOpenGLShader;
use crate::rendering::shader::OpenGLShader;
use crate::rendering::{OpenGLRenderer, PrimitiveRenderer, RenderContext};
use crate::rendering::camera::OrthographicCamera;
use crate::rendering::post::{OpenGLPostProcessRenderer, OpenGLPostProcessShader, RenderTarget};
use crate::ui::rendering::WideRenderContext;
use crate::ui::styles::InheritSupplier;
use crate::window::Window;

enum Post {
    None,
    Some(PostSources)
}

struct PostSources {
    renderer: OpenGLPostProcessRenderer,
    target: Option<RenderTarget>,
    shaders: Vec<OpenGLPostProcessShader>,
    index: usize
}

impl PostSources {
    fn new(window: &Window) -> Self {
        Self {
            renderer: OpenGLPostProcessRenderer::new(window.width(), window.height()),
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
        })
    }

    pub fn create_post(&mut self, window: &Window) {
        self.post = Post::Some(PostSources::new(window))
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
        if let Post::Some(sources) = &mut self.post {
            sources.renderer = OpenGLPostProcessRenderer::new(window.width(), window.height());
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
    pub fn advance<F: Fn(&mut OpenGLShader)>(&mut self, window: &Window, f: F) {
        if let Post::Some(sources) = &mut self.post {
            if !self.rendered {
                let target = self.controller.draw_to_target(window, &self.camera, &mut self.renderer, &mut self.shader);
                sources.renderer.set_target(target);
                self.rendered = true;
            } else {
                if sources.index >= sources.shaders.len() {
                    warn!("Illegal call to advance() on RenderingPipeline as there are no more shaders to process!");
                } else {
                    let shader = &mut sources.shaders[sources.index];
                    shader.use_program();
                    f(shader);
                    trace!("Running shader in pipeline");
                    sources.renderer.run_shader(shader);
                    sources.index += 1;
                    //Idk i am more comfortable with >= than == and does it even matter
                    if sources.index >= sources.shaders.len() {
                        sources.renderer.draw_to_screen();
                        trace!("Drew pipeline to screen.");
                    }
                }
            }
        } else {
            if !self.rendered {
                self.controller.draw(window, &self.camera, &mut self.renderer, &mut self.shader);
                trace!("Drew pipeline to screen.");
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
                    sources.renderer.draw_to_screen();
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
        other.begin_frame();
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