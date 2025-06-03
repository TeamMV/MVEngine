use std::sync::Arc;
use crate::audio::source::Sound;

pub mod wav;

pub trait AudioDecoder {
    fn is_compatible(input: &[u8]) -> bool;
    fn decode(&self, input: &[u8]) -> Arc<Sound>;
}