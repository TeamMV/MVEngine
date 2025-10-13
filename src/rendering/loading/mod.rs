pub mod gltf;
pub mod obj;

#[derive(Clone, Debug)]
pub enum ModelLoadingError {
    MissingFile(String),
    IllegalContent(String),
    FailedToFetch(String),
    UnexpectedEndOfFile,
}
