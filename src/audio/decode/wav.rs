use crate::audio::decode::Decoder;

pub struct WavDecoder;

impl Decoder for WavDecoder {
    fn read(bytes: &[u8]) -> Vec<f32> {
        todo!()
    }
}