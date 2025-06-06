use crate::math::vec::Vec4;
use gl::types::{GLint, GLsizei, GLuint};
use image::{GenericImageView, ImageError};
use mvutils::save::{Loader, Savable, Saver};
use mvutils::utils::TetrahedronOp;
use mvutils::Savable;

#[derive(Clone)]
pub struct Texture {
    pub id: GLuint,
    pub dimensions: (u32, u32),
    sampler: bool
}

impl Texture {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ImageError> {
        Self::from_bytes_sampled(bytes, false)
    }

    pub fn from_bytes_sampled(bytes: &[u8], smooth: bool) -> Result<Self, ImageError> {
        let img = image::load_from_memory(bytes)?;
        let (width, height) = img.dimensions();

        let img = img.flipv();
        let img = img.to_rgba8();

        let mut texture_id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);

            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                smooth.yn(gl::LINEAR, gl::NEAREST) as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                smooth.yn(gl::LINEAR, gl::NEAREST) as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as GLint,
            );

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
        }

        Ok(Self {
            id: texture_id,
            dimensions: (width, height),
            sampler: smooth,
        })
    }

    pub fn from_rgba(data: &[u8], smooth: bool) -> Result<Self, String> {
        if data.len() < 8 {
            return Err("Data too short for RGBA texture".into());
        }

        let width = u32::from_le_bytes(data[0..4].try_into().unwrap());
        let height = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let pixel_data = &data[8..];

        if pixel_data.len() != (width * height * 4) as usize {
            return Err("Pixel data size does not match dimensions".into());
        }

        let mut texture_id: GLuint = 0;

        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, smooth.yn(gl::LINEAR, gl::NEAREST) as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, smooth.yn(gl::LINEAR, gl::NEAREST) as GLint);
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
                pixel_data.as_ptr() as *const _,
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        Ok(Self {
            id: texture_id,
            dimensions: (width, height),
            sampler: smooth,
        })
    }

    pub fn get_uv(&self) -> [(f32, f32); 4] {
        [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
    }

    pub fn get_uv_inner(&self, outer_uv: Vec4) -> [(f32, f32); 4] {
        [(outer_uv.x, outer_uv.y), (outer_uv.x + outer_uv.z, outer_uv.y), (outer_uv.x + outer_uv.z, outer_uv.y + outer_uv.w), (outer_uv.x, outer_uv.y + outer_uv.w)]
    }
}

impl Savable for Texture {
    fn save(&self, saver: &mut impl Saver) {
        self.sampler.save(saver);

        let (width, height) = self.dimensions;
        let mut raw = vec![0u8; (width * height * 4) as usize];

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
            gl::GetTexImage(
                gl::TEXTURE_2D,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                raw.as_mut_ptr() as *mut _,
            );
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        let mut final_data = Vec::with_capacity(8 + raw.len());
        final_data.extend(&width.to_le_bytes());
        final_data.extend(&height.to_le_bytes());
        final_data.extend(raw);

        final_data.save(saver);
    }

    fn load(loader: &mut impl Loader) -> Result<Self, String> {
        let sampler = bool::load(loader)?;
        let bytes = Vec::<u8>::load(loader)?;
        Self::from_rgba(bytes.as_slice(), sampler).map_err(|x| x.to_string())
    }
}

#[derive(Savable)]
pub struct NoCtxTexture {
    raw_data: Vec<u8>,
    smooth: bool
}

impl NoCtxTexture {
    pub fn new(data: &[u8], smooth: bool) -> Self {
        Self {
            raw_data: data.to_vec(),
            smooth,
        }
    }
}

impl TryFrom<NoCtxTexture> for Texture {
    type Error = ImageError;

    fn try_from(value: NoCtxTexture) -> Result<Self, Self::Error> {
        Self::from_bytes_sampled(&value.raw_data, value.smooth)
    }
}