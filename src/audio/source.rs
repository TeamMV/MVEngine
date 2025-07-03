use crate::audio::decode::wav::ActuallyUsefulWavData;
use crate::ui::ease::{Easing, EasingGen, EasingMode};
use mvutils::unsafe_utils::DangerousCell;
use std::sync::Arc;

// "Fucl u" - v22 (4/6/25)
// "Fucl you" - v22 (5/6/25)
// fucl YOU max - v22 right now
pub struct SoundWithAttributes {
    sound: Arc<Sound>,
    with_attributes: WithAttributes,
}

impl SoundWithAttributes {
    pub fn new(sound: Arc<Sound>) -> Arc<Self> {
        let this = Self {
            sound,
            with_attributes: WithAttributes::new(),
        };
        Arc::new(this)
    }

    pub fn map_index(&self, index: usize, sample_rate: u32) -> usize {
        (self.sound.map_index(index, sample_rate) as f32 * self.with_attributes.speed.get_val())
            .round() as usize
    }

    pub fn get_sample_mapped(&self, index: usize, sample_rate: u32) -> (f32, f32) {
        self.get_sample(self.map_index(index, sample_rate))
    }

    pub fn get_sample(&self, index: usize) -> (f32, f32) {
        if self.sound.samples.is_empty() {
            return (0.0, 0.0);
        }

        let looping = self.with_attributes.looping.get_val();
        let mut volume: f32 = self.with_attributes.volume.get_val();

        match self.sound.channels {
            2 => {
                let sample_count = self.sound.samples.len() / 2;
                let idx = if looping {
                    index % sample_count
                } else {
                    if index >= sample_count {
                        return (0.0, 0.0);
                    }
                    index
                };
                volume *= self.with_attributes.get_easing_multiplier(idx);
                (
                    self.sound.samples[idx * 2] * volume,
                    self.sound.samples[idx * 2 + 1] * volume,
                )
            }
            1 => {
                let balance = self.with_attributes.mono_balance.get_val();
                let sample_count = self.sound.samples.len();
                let idx = if looping {
                    index % sample_count
                } else {
                    if index >= sample_count {
                        return (0.0, 0.0);
                    }
                    index
                };
                volume *= self.with_attributes.get_easing_multiplier(idx);
                let sample = self.sound.samples[idx];
                (sample * (1.0 - balance) * volume, sample * balance * volume)
            }
            _ => {
                panic!("We do not support futuristic 3+ ear headphones");
            }
        }
    }

    pub fn is_looping(&self) -> bool {
        self.with_attributes.looping.get_val()
    }

    pub fn set_looping(&self, looping: bool) {
        self.with_attributes.looping.replace(looping);
    }

    pub fn balance(&self) -> f32 {
        self.with_attributes.mono_balance.get_val()
    }

    pub fn set_balance(&self, balance: f32) {
        self.with_attributes.mono_balance.replace(balance);
    }

    pub fn volume(&self) -> f32 {
        self.with_attributes.volume.get_val()
    }

    pub fn set_volume(&self, volume: f32) {
        self.with_attributes.volume.replace(volume);
    }

    pub fn speed(&self) -> f32 {
        self.with_attributes.speed.get_val()
    }

    pub fn set_speed(&self, speed: f32) {
        self.with_attributes.speed.replace(speed);
    }
    
    pub fn set_fade_in(&self, ease_gen: EasingGen, duration_ms: u32) {
        let duration_samples = (duration_ms * self.sound.sample_rate) / 1000;
        self.with_attributes.fade_in.replace(Some(Easing::new(
            ease_gen,
            EasingMode::In,
            0.0..duration_samples as f32,
            0.0..1.0,
        )));
    }

    pub fn set_fade_out(&self, ease_gen: EasingGen, duration_ms: u32) {
        let duration_samples = (duration_ms * self.sound.sample_rate) / 1000;
        let start_sample = self.sound.effective_samples() as u32 - duration_samples;
        self.with_attributes.fade_out.replace(Some(Easing::new(
            ease_gen,
            EasingMode::Out,
            start_sample as f32..(start_sample + duration_samples) as f32,
            1.0..0.0,
        )));
    }

    pub fn sound(&self) -> Arc<Sound> {
        self.sound.clone()
    }

    pub fn with_attributes(&self) -> &WithAttributes {
        &self.with_attributes
    }

    pub fn full_clone(self: &Arc<Self>) -> Arc<Self> {
        let this = Self {
            sound: self.sound.clone(),
            with_attributes: self.with_attributes.clone(),
        };
        Arc::new(this)
    }
}

unsafe impl Send for SoundWithAttributes {}
unsafe impl Sync for SoundWithAttributes {}

pub struct WithAttributes {
    // Fuck memory safety, we need to read this shit 48000 times a second we dont have time for rwlock overhead
    mono_balance: DangerousCell<f32>,
    volume: DangerousCell<f32>,
    speed: DangerousCell<f32>,
    looping: DangerousCell<bool>,

    fade_in: DangerousCell<Option<Easing>>,
    fade_out: DangerousCell<Option<Easing>>,
}

impl WithAttributes {
    pub fn new() -> Self {
        Self {
            mono_balance: 0.5.into(),
            volume: 1.0.into(),
            speed: 1.0.into(),
            looping: false.into(),

            fade_in: None.into(),
            fade_out: None.into(),
        }
    }

    pub fn get_easing_multiplier(&self, index: usize) -> f32 {
        let fade_in = if let Some(fade_in) = self.fade_in.get() {
            fade_in.get(index as f32)
        } else {
            1.0
        };
        let fade_out = if let Some(fade_out) = self.fade_out.get() {
            fade_out.get(index as f32)
        } else {
            1.0
        };
        fade_in * fade_out
    }

    pub fn is_looping(&self) -> bool {
        self.looping.get_val()
    }

    pub fn set_looping(&self, looping: bool) {
        self.looping.replace(looping);
    }

    pub fn balance(&self) -> f32 {
        self.mono_balance.get_val()
    }

    pub fn set_balance(&self, balance: f32) {
        self.mono_balance.replace(balance);
    }

    pub fn volume(&self) -> f32 {
        self.volume.get_val()
    }

    pub fn set_volume(&self, volume: f32) {
        self.volume.replace(volume);
    }

    pub fn speed(&self) -> f32 {
        self.speed.get_val()
    }

    pub fn set_speed(&self, speed: f32) {
        self.speed.replace(speed);
    }
}

impl Clone for WithAttributes {
    fn clone(&self) -> Self {
        Self {
            mono_balance: self.mono_balance.get_val().into(),
            volume: self.volume.get_val().into(),
            speed: self.speed.get_val().into(),
            looping: self.looping.get_val().into(),
            fade_in: self.fade_in.get().clone().into(),
            fade_out: self.fade_out.get().clone().into(),
        }
    }
}

pub struct Sound {
    channels: u8,
    sample_rate: u32,
    samples: Vec<f32>,
}

impl Sound {
    pub fn from_wav(data: ActuallyUsefulWavData) -> Arc<Self> {
        Self::from_raw(data.channels, data.sample_rate, data.samples)
    }

    pub fn from_raw(channels: u8, sample_rate: u32, samples: Vec<f32>) -> Arc<Self> {
        let this = Self {
            channels,
            sample_rate,
            samples,
        };
        Arc::new(this)
    }

    pub fn total_samples(&self) -> usize {
        self.samples.len()
    }

    pub fn effective_samples(&self) -> usize {
        self.samples.len() / self.channels as usize
    }

    pub fn map_index(&self, index: usize, sample_rate: u32) -> usize {
        let ratio = self.sample_rate as f32 / sample_rate as f32;
        (index as f32 * ratio).round() as usize
    }

    pub fn get_sample_raw(&self, index: usize) -> f32 {
        self.samples[index]
    }

    pub fn get_sample_mapped(&self, index: usize, sample_rate: u32) -> (f32, f32) {
        self.get_sample(self.map_index(index, sample_rate))
    }

    pub fn get_sample(&self, index: usize) -> (f32, f32) {
        if self.samples.is_empty() {
            return (0.0, 0.0);
        }
        match self.channels {
            2 => (self.samples[index * 2], self.samples[index * 2 + 1]),
            1 => {
                let sample = self.samples[index];
                (sample * 0.5, sample * 0.5)
            }
            _ => {
                panic!("We do not support futuristic 3+ ear headphones");
            }
        }
    }

    pub fn channels(&self) -> u8 {
        self.channels
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}
