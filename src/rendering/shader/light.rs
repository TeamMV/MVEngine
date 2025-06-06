use crate::rendering::shader::OpenGLShader;
use std::ops::{Deref, DerefMut};

#[repr(transparent)]
pub struct LightOpenGLShader(OpenGLShader);

impl LightOpenGLShader {
    pub fn new() -> Self {
        Self(OpenGLShader::new(
            include_str!("../shaders/index.vert"),
            include_str!("../shaders/light.frag"),
        ))
    }
}

impl Deref for LightOpenGLShader {
    type Target = OpenGLShader;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LightOpenGLShader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
