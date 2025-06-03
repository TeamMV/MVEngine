pub mod mixer;
pub mod source;
pub mod decode;

use std::f32::consts::PI;
use std::sync::Arc;
use cpal::{BufferSize, ChannelCount, SampleRate, Stream, StreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::{error, info};
use parking_lot::Mutex;
use crate::audio::mixer::AudioMixer;
use crate::audio::source::Sound;

pub struct AudioEngine {
    mixer: Arc<Mutex<AudioMixer>>,
    stream: Stream,
    sample_rate: u32
}

impl AudioEngine {
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

impl AudioEngine {
    pub fn setup() -> Option<Self> {
        let host = cpal::default_host();
        if let Some(device) = host.default_output_device() {
            info!("Selected audio device: {:?}", device.name());

            let mut index = 0;
            let config = device.default_output_config();
            if let Ok(config) = config {
                let mixer = Arc::new(Mutex::new(AudioMixer::new()));
                let cloned_mixer = mixer.clone();

                let sample_rate = config.sample_rate().0;
                
                let stream = device.build_output_stream(&config.config(), move |data: &mut [f32], _| {
                    for sample in data.chunks_mut(config.channels() as usize) {
                        let tone = mixer.lock().get_current_sample(index);

                        sample[0] = tone.0;
                        sample[1] = tone.1;

                        index = index.wrapping_add(1);
                    }
                }, |e| {
                    error!("Error during audio playback: {e}");
                }, None);

                if let Ok(stream) = stream {
                    stream.play().ok()?;
                    return Some(Self {
                        stream,
                        mixer: cloned_mixer,
                        sample_rate
                    })
                } else {
                    error!("Error playing stream")
                }
            } else {
                error!("Invalid audio config");
            }
        }


        None
    }
    
    pub fn play_sound(&self, sound: Arc<Sound>) {
        self.mixer.lock().play(sound)
    }
}

fn gen_tone(freq: u32, sample_idx: u32, sample_rate: u32) -> f32 {
    let freq = freq as f32;
    let t = sample_idx as f32 / sample_rate as f32;
    (2.0 * PI * freq * t).sin()
}

pub fn gen_sin_wave(freq: u32, rate: u32, duration: u32) -> Arc<Sound> {
    let total_samples = ((duration as f32 / 1000.0) * rate as f32) as usize;
    let freq = freq as f32;
    let rate = rate as f32;

    let mut samples = Vec::with_capacity(total_samples);
    for n in 0..total_samples {
        let t = n as f32 / rate;
        let sample = (2.0 * PI * freq * t).sin();
        samples.push(sample);
    }
    
    Sound::from_raw(1, rate as u32, samples)
}