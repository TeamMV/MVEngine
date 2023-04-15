pub mod assets;
pub mod render;
pub mod parser;
pub mod math;

#[cfg(test)]
mod tests {
    use crate::render::Renderer;
    use crate::render::RenderingBackend::OpenGL;
    use crate::render::shared::*;

    #[test]
    fn it_works() {
        let mut renderer = Renderer::new(OpenGL);
        let info = WindowCreateInfo::default();
        let mut window = renderer.create_window(info);
        window.run_default();
    }
}
