pub mod assets;
pub mod render;
pub mod parser;

#[cfg(test)]
mod tests {
    use cgmath::{Vector4, Zero};

    use crate::render::RenderCore;
    use crate::render::RenderingBackend::OpenGL;
    use crate::render::shared::*;

    #[test]
    fn it_works() {
        let mut renderer = RenderCore::new(OpenGL);
        let info = WindowCreateInfo::default();
        let mut window = renderer.create_window(info);
        window.run_default();
    }
}
