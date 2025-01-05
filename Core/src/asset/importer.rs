use image::{ColorType, DynamicImage};
use std::fs::OpenOptions;
use std::io::Read;
use crate::asset::manager::AssetHandle;

use crate::render::backend::buffer::MemoryProperties;
use crate::render::backend::device::Device;
use crate::render::backend::image::{
    Image, ImageAspect, ImageTiling, ImageType, ImageUsage, MVImageCreateInfo,
};
use crate::render::backend::Extent2D;
use crate::render::texture::Texture;

#[derive(Clone)]
pub struct AssetLoader {
    device: Device,
}

impl AssetLoader {
    pub fn new(device: Device) -> Self {
        Self { device }
    }

    // pub(crate) fn import_model(&self, gltf: gltf::Result<Gltf>) -> Model {
    //     let gltf = gltf.unwrap_or_else(|e| {
    //         log::error!("Failed to load model, path:, error: {e}");
    //         panic!();
    //     });
    //
    //     // for scene in gltf.scenes() {
    //     //     Self::process_node(gltf, scene.nodes().nth(0).unwrap_or_else(|e| {
    //     //         log::error!("Failed to get root node? this should never happen, error: {e}");
    //     //         panic!();
    //     //     }));
    //     // }
    //
    //     todo!();
    // }
    //
    // fn process_node(&self, gltf: Gltf, node: gltf::Node) -> (Vec<Mesh>, Vec<String>, Vec<Texture>, Vec<Material>) {
    //
    //     let transform = node.transform();
    //     for mesh in node.mesh() {
    //         for primitive in mesh.primitives() {
    //             let pos = primitive.get(&Semantic::Positions).unwrap().view().unwrap();
    //             let data = match pos.buffer().source() {
    //                 Source::Bin => &gltf.blob.as_ref().unwrap()[pos.offset()..pos.offset() + pos.length()],
    //                 Source::Uri(file) => panic!(),
    //             };
    //         }
    //     }
    //
    //     todo!();
    // } ignore that for now, lets load in textures

    pub(crate) fn import_texture(&self, handle: AssetHandle) -> Result<Texture, &'static str> {
        let path = handle.get_path();
        let image = if let Ok(image) = image::open(path) {
            image
        } else {
            let mut buffer = Vec::new();
            let mut file = OpenOptions::new()
                .read(true)
                .open(path)
                .map_err(|_| "Failed to open texture file")?;
            file.read_to_end(&mut buffer)
                .map_err(|_| "Failed to read texture file")?;
            image::load_from_memory(&buffer).map_err(|_| "Invalid texture format")?
        };

        let image = match image.color() {
            ColorType::L8 => DynamicImage::ImageRgba8(image.to_rgba8()),
            ColorType::La8 => DynamicImage::ImageRgba8(image.to_rgba8()),
            ColorType::Rgb8 => DynamicImage::ImageRgba8(image.to_rgba8()),
            ColorType::Rgba8 => image,
            ColorType::L16 => DynamicImage::ImageRgba16(image.to_rgba16()),
            ColorType::La16 => DynamicImage::ImageRgba16(image.to_rgba16()),
            ColorType::Rgb16 => DynamicImage::ImageRgba16(image.to_rgba16()),
            ColorType::Rgba16 => image,
            ColorType::Rgb32F => DynamicImage::ImageRgba32F(image.to_rgba32f()),
            ColorType::Rgba32F => image,
            _ => image,
        };

        // TODO: check if this was an issue in the shader, remove this if it was
        //let image = image.fliph();

        let width = image.width();
        let height = image.height();
        let format = image.color();
        let data = image.into_bytes();

        let image = Image::new(
            self.device.clone(),
            MVImageCreateInfo {
                size: Extent2D { width, height },
                format: format.into(),
                usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED | ImageUsage::STORAGE,
                memory_properties: MemoryProperties::DEVICE_LOCAL,
                aspect: ImageAspect::COLOR,
                tiling: ImageTiling::Optimal,
                layer_count: 1,
                image_type: ImageType::Image2D,
                cubemap: false,
                memory_usage_flags: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
                data: Some(data),
                label: Some(path.to_string()),
            },
        );

        Ok(Texture::new(image, handle))
    }
}
