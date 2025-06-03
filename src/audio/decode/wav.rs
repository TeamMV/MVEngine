use std::sync::Arc;
use bytebuffer::{ByteBuffer, Endian};
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::Savable;
use mvutils::save::custom::{raw_vec_save, raw_vec_load};
use mvutils::save::{Loader, Savable};
use crate::audio::decode::AudioDecoder;
use crate::audio::source::Sound;

pub struct WavDecoder;

pub type Static4String = [u8; 4];

#[derive(Debug, Savable)]
pub struct WavData {
    header: Static4String,
    file_size: u32,
    WAVE: Static4String,
    fmt_null_byte: Static4String,
    format_length: u32,
    format_type: i16,
    num_channels: u16,
    sample_rate: u32,
    weird_calculation_using_data_already_in_this_struct: u32,
    even_uselessler_calculation: u16,
    bits_per_sample: u16,
    data_header: Static4String,
    data_size: u32,
    #[custom(save = raw_vec_save, load = raw_vec_load)]
    data: Vec<u8>,
}

impl WavData {
    // NOTE: idk if this works lmao
    pub fn validate(&self) -> bool {
        self.header == *b"RIFF" &&
            self.WAVE == *b"WAVE" &&
            self.fmt_null_byte == *b"fmt\0"
    }
}

pub struct ActuallyUsefulWavData {
    pub channels: u8,
    pub sample_rate: u32,
    pub samples: Vec<f32>,
}

impl From<WavData> for ActuallyUsefulWavData {
    fn from(value: WavData) -> Self {
        let mut buffer = ByteBuffer::from_vec_le(value.data);
        let mut samples = Vec::new();
        match value.bits_per_sample {
            8 => {
                while let Some(value) = buffer.pop_u8() {
                    samples.push(((value as f32) - 128.0) / 128.0);
                }
            }
            16 => {
                while let Some(value) = buffer.pop_i16() {
                    samples.push((value as f32) / 32768.0);
                }
            }
            _ => {
                panic!("Illegal wav format, you are going to jail");
            }
        }
        ActuallyUsefulWavData {
            channels: value.num_channels as u8,
            sample_rate: value.sample_rate,
            samples,
        }
    }
}

impl AudioDecoder for WavDecoder {
    fn is_compatible(input: &[u8]) -> bool {
        let mut buffer = ByteBuffer::from_bytes(input);
        buffer.set_endian(Endian::LittleEndian);
        let decoded = WavData::load(&mut buffer);
        if let Ok(decoded) = decoded {
            decoded.validate()
        } else {
            false
        }
    }

    fn decode(&self, input: &[u8]) -> Arc<Sound> {
        let mut buffer = ByteBuffer::from_bytes(input);
        buffer.set_endian(Endian::LittleEndian);
        let decoded = WavData::load(&mut buffer).expect("Not wav data");
        Sound::from_wav(decoded.into())
    }
}