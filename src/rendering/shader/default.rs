use std::ops::{Deref, DerefMut};
use crate::rendering::shader::OpenGLShader;

#[repr(transparent)]
pub struct DefaultOpenGLShader(OpenGLShader);

impl DefaultOpenGLShader {
    pub fn new() -> Self {
        Self (OpenGLShader::new(
            include_str!("../shaders/index.vert"),
            include_str!("../shaders/index.frag"),
        ))
    }
}

impl Deref for DefaultOpenGLShader {
    type Target = OpenGLShader;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DefaultOpenGLShader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}