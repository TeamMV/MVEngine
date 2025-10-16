pub mod v2;

use crate::rendering::implementation::scene::Scene;
use crate::rendering::loading::ModelLoadingError;
use bytebuffer::{ByteBuffer, Endian};
use mvutils::Savable;
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::save::{Loader, Savable, Saver};

pub struct GLTFModelLoader {
    scenes: Vec<Scene>,
}

impl GLTFModelLoader {
    pub fn load(data: &[u8]) -> Result<Self, ModelLoadingError> {
        let mut loader = ByteBuffer::from_bytes(data);
        loader.set_endian(Endian::LittleEndian);
        let header = GLTFHeader::load(&mut loader).map_err(ModelLoadingError::FailedToFetch)?;

        if header.magic != 0x46546C67 {
            return Err(ModelLoadingError::IllegalContent(format!(
                "{} are not the glTF magic bytes! You cannot trick us!",
                header.magic
            )));
        }

        let scenes = match header.version {
            2 => v2::load_scenes(header, &mut loader)?,
            _ => {
                return Err(ModelLoadingError::IllegalContent(
                    "MVEngine can currently only load glTF version 2!".to_string(),
                ));
            }
        };

        Ok(Self { scenes })
    }
}

#[derive(Savable)]
struct GLTFHeader {
    magic: u32,
    version: u32,
    length: u32,
}

enum ChunkType {
    Json,
    Bin,
    Unknown,
}

impl Savable for ChunkType {
    fn save(&self, saver: &mut impl Saver) {
        unimplemented!()
    }

    fn load(loader: &mut impl Loader) -> Result<Self, String> {
        let id = u32::load(loader)?;
        match id {
            0x4E4F534A => Ok(ChunkType::Json),
            0x004E4942 => Ok(ChunkType::Bin),
            _ => Ok(ChunkType::Unknown),
        }
    }
}

struct GLTFChunk {
    pub(crate) chunk_type: ChunkType,
    pub(crate) content: Vec<u8>,
}

impl Savable for GLTFChunk {
    fn save(&self, saver: &mut impl Saver) {
        unimplemented!()
    }

    fn load(loader: &mut impl Loader) -> Result<Self, String> {
        let length = u32::load(loader)?;
        let ty = ChunkType::load(loader)?;
        let data = loader
            .pop_bytes(length as usize)
            .ok_or("Cannot pop bytes from loader!".to_string())?;
        Ok(Self {
            chunk_type: ty,
            content: data,
        })
    }
}
