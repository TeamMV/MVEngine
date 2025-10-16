use mvengine_ui_parsing::json::from_json::FromJsonError;
use crate::utils::decode::Base64DecodeError;

pub mod gltf;
pub mod obj;

#[derive(Clone, Debug)]
pub enum ModelLoadingError {
    MissingFile(String),
    IllegalContent(String),
    FailedToFetch(String),
    UnexpectedEndOfFile,
    IllegalJson(FromJsonError),
    IllegalBase64(Base64DecodeError)
}
