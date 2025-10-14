mod bin;

use crate::rendering::implementation::scene::Scene;
use crate::rendering::loading::ModelLoadingError;
use crate::rendering::loading::gltf::{ChunkType, GLTFChunk, GLTFHeader};
use bytebuffer::ByteBuffer;
use mvengine_ui_parsing::json::{JsonIdentFlavor, parse_json};
use mvutils::save::Savable;
use mvengine_ui_parsing::json::from_json::FromJsonTrait;
use crate::rendering::loading::gltf::v2::bin::GLTFFile;

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

}

fn load_chunk(loader: &mut ByteBuffer) -> Result<GLTFChunk, ModelLoadingError> {
    GLTFChunk::load(loader).map_err(ModelLoadingError::FailedToFetch)
}
