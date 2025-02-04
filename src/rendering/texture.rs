use gl::types::{GLint, GLsizei, GLuint, GLuint64};
use image::{GenericImageView, ImageError};
use crate::rendering::bindless;

#[derive(Clone)]
pub struct Texture {
    pub id: GLuint,
    pub handle: GLuint64
}

impl Texture {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ImageError> {
        let img = image::load_from_memory(bytes)?;
        let (width, height) = img.dimensions();

        let img = img.flipv();
        let img = img.to_rgba8();

        let mut texture_id: GLuint = 0;
        let mut handle = 0;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);

            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                width as GLsizei,
                height as GLsizei,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.as_ptr() as *const _,
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);

            handle = 0;//bindless::GetTextureHandleARB(texture_id);
        }

        Ok(Self { id: texture_id, handle })
    }

    pub fn get_uv(&self) -> [(f32, f32); 4] {
        [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
    }
}