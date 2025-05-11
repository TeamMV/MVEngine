mod wav;

pub trait Decoder {
    fn read(bytes: &[u8]) -> Vec<f32>;
}