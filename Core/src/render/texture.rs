use crate::asset::manager::AssetHandle;
use crate::math::vec::Vec4;
use crate::render::backend::image::Image;

#[derive(Clone)]
pub struct Texture {
    image: Image,
    handle: AssetHandle,
    id: u64,
}

impl Texture {
    pub fn new(image: Image, handle: AssetHandle) -> Self {
        Self {
            image,
            handle,
            id: mvutils::utils::next_id("MVCore::Texture"),
        }
    }

    pub fn as_region(&self, x: u32, y: u32, width: u32, height: u32) -> TextureRegion {
        TextureRegion::new(
            self.clone(),
            x,
            self.image.get_extent().height - y - height,
            width,
            height,
        )
    }

    pub fn as_full_region(&self) -> TextureRegion {
        TextureRegion {
            texture: self.clone(),
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }
    }

    pub fn image(&self) -> Image {
        self.image.clone()
    }
}

impl PartialEq for Texture {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(PartialEq, Clone)]
pub struct TextureRegion {
    texture: Texture, // we can clone dw
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl TextureRegion {
    pub fn new(texture: Texture, x: u32, y: u32, width: u32, height: u32) -> Self {
        let extent = texture.image.get_extent();
        let w = extent.width as f32;
        let h = extent.height as f32;
        Self {
            texture,
            x: x as f32 / w,
            y: y as f32 / h,
            width: width as f32 / w,
            height: height as f32 / h,
        }
    }

    pub fn same_texture(&self, other: &TextureRegion) -> bool {
        self.texture == other.texture
    }

    pub fn coords(&self) -> Vec4 {
        Vec4::new(self.x, self.y, self.width, self.height)
    }
}
