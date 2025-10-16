mod json;
mod bin;

use std::slice;
use crate::rendering::implementation::scene::Scene;
use crate::rendering::loading::ModelLoadingError;
use crate::rendering::loading::gltf::{ChunkType, GLTFChunk, GLTFHeader};
use bytebuffer::ByteBuffer;
use mvengine_ui_parsing::json::{JsonIdentFlavor, parse_json};
use mvutils::save::Savable;
use mvengine_ui_parsing::json::from_json::FromJsonTrait;
use crate::rendering::loading::gltf::v2::bin::Buffer;
use crate::rendering::loading::gltf::v2::json::{GLTFBuffer, GLTFFile};
use crate::utils::decode;

pub fn load_scenes(
    header: GLTFHeader,
    loader: &mut ByteBuffer,
) -> Result<Vec<Scene>, ModelLoadingError> {
    let probably_json_chunk = load_chunk(loader)?;
    if !matches!(probably_json_chunk.chunk_type, ChunkType::Json) {
        return Err(ModelLoadingError::IllegalContent(
            "The first chunk of a glTF 2.0 file MUST be JSON!".to_string(),
        ));
    }

    let mut chunks = vec![];
    while loader.get_rpos() < header.length as usize {
        let chunk = load_chunk(loader)?;
        chunks.push(chunk);
    }

    let raw_json = str::from_utf8(&probably_json_chunk.content)
        .map_err(|e| ModelLoadingError::IllegalContent(e.to_string()))?;
    let json = parse_json(raw_json, JsonIdentFlavor::Identifiers)
        .map_err(|e| ModelLoadingError::IllegalContent(e))?;

    let gltf = GLTFFile::from_json(&json)
        .map_err(ModelLoadingError::IllegalJson)?;


    let buffers = gltf.buffers
        .iter()
        .enumerate()
        .map(|(index, x)| {
            let b = acquire_buffer(x, index, &chunks);
        })
}

fn acquire_buffer<'a>(json: &GLTFBuffer, index: usize, chunks: Vec<GLTFChunk>) -> Result<Buffer<'a>, ModelLoadingError> {
    if let Some(uri) = json.uri {
        if let Some(data_thing) = uri.strip_prefix("data:") {
            if let Some(idx) = data_thing.find("base64,") {
                let idx = idx + 7;
                let base64 = &data_thing[idx..];
                let raw = decode::decode_base64(base64);
            }
        }
    } else {
        //assume the buffer is inside the glb file
        //according to gltf spec, there can only be 1 BIN chunk and thats always the first buffer in the array
        let chunk = &chunks[index + 1];
        //assume chunk is BIN
        Buffer {
            slice: chunk.content.as_slice(),
        }
    }
}

fn load_chunk(loader: &mut ByteBuffer) -> Result<GLTFChunk, ModelLoadingError> {
    GLTFChunk::load(loader).map_err(ModelLoadingError::FailedToFetch)
}
