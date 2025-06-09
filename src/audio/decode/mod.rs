use crate::audio::source::Sound;
use std::sync::Arc;

pub mod wav;

pub trait AudioDecoder {
    fn is_compatible(input: &[u8]) -> bool;
    fn decode(&self, input: &[u8]) -> Arc<Sound>;
}
