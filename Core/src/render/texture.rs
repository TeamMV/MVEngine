use crate::render::backend::image::Image;

#[derive(Clone)]
pub struct Texture {
    image: Image,
    id: u64,
}

impl Texture {
    pub fn new(image: Image) -> Self {
        Self {
            image,
            id: mvutils::utils::next_id("MVCore::Texture"),
        }
    }

    pub fn as_region(&self, x: u32, y: u32, width: u32, height: u32) -> TextureRegion {
        TextureRegion::new(self.clone(), x, y, width, height)
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
}