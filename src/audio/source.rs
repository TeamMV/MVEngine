use std::mem;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use crate::audio::decode::wav::ActuallyUsefulWavData;

// "Fucl u" - v22
pub struct SoundWithAttributes {
    // hacky ass workaround, use raw bytes lmao
    mono_balance: AtomicU32,
    volume: AtomicU32,
    speed: AtomicU32,
    looping: AtomicBool,
    sound: Arc<Sound>
}

impl SoundWithAttributes {
    pub fn new(sound: Arc<Sound>) -> Arc<Self> {
        let this = Self {
            mono_balance: AtomicU32::new(unsafe { mem::transmute(0.5f32) }),
            volume: AtomicU32::new(unsafe { mem::transmute(1.0f32) }),
            speed: AtomicU32::new(unsafe { mem::transmute(1.0f32) }),
            looping: AtomicBool::new(false),
            sound
        };
        Arc::new(this)
    }

    pub fn map_index(&self, index: usize, sample_rate: u32) -> usize {
        let speed: f32 = unsafe { mem::transmute(self.speed.load(Ordering::Acquire)) };
        (self.sound.map_index(index, sample_rate) as f32 * speed).round() as usize
    }

    pub fn get_sample_mapped(&self, index: usize, sample_rate: u32) -> (f32, f32) {
        self.get_sample(self.map_index(index, sample_rate))
    }

    pub fn get_sample(&self, index: usize) -> (f32, f32) {
        if self.sound.samples.is_empty() {
            return (0.0, 0.0);
        }

        let looping = self.looping.load(Ordering::Acquire);
        let volume: f32 = unsafe { mem::transmute(self.volume.load(Ordering::Acquire)) };

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
                (self.sound.samples[idx * 2] * volume, self.sound.samples[idx * 2 + 1] * volume)
            }
            1 => {
                let balance = unsafe { mem::transmute::<u32, f32>(self.mono_balance.load(Ordering::Acquire)) };
                let sample_count = self.sound.samples.len();
                let idx = if looping {
                    index % sample_count
                } else {
                    if index >= sample_count {
                        return (0.0, 0.0);
                    }
                    index
                };
                let sample = self.sound.samples[idx];
                (sample * (1.0 - balance) * volume, sample * balance * volume)
            }
            _ => {
                panic!("We do not support futuristic 3+ ear headphones");
            }
        }
    }

    pub fn is_looping(&self) -> bool {
        self.looping.load(Ordering::Acquire)
    }

    pub fn set_looping(&self, looping: bool) {
        self.looping.store(looping, Ordering::Release);
    }

    pub fn balance(&self) -> f32 {
        unsafe { mem::transmute(self.mono_balance.load(Ordering::Acquire)) }
    }

    pub fn set_balance(&self, balance: f32) {
        self.mono_balance.store(unsafe { mem::transmute(balance) }, Ordering::Release);
    }

    pub fn volume(&self) -> f32 {
        unsafe { mem::transmute(self.volume.load(Ordering::Acquire)) }
    }

    pub fn set_volume(&self, volume: f32) {
        self.volume.store(unsafe { mem::transmute(volume) }, Ordering::Release);
    }

    pub fn speed(&self) -> f32 {
        unsafe { mem::transmute(self.speed.load(Ordering::Acquire)) }
    }

    pub fn set_speed(&self, speed: f32) {
        self.speed.store(unsafe { mem::transmute(speed) }, Ordering::Release);
    }

    pub fn sound(&self) -> Arc<Sound> {
        self.sound.clone()
    }
    
    pub fn full_clone(self: &Arc<Self>) -> Arc<Self> {
        let this = Self {
            mono_balance: AtomicU32::new(self.mono_balance.load(Ordering::Acquire)),
            volume: AtomicU32::new(self.volume.load(Ordering::Acquire)),
            speed: AtomicU32::new(self.speed.load(Ordering::Acquire)),
            looping: AtomicBool::new(self.looping.load(Ordering::Acquire)),
            sound: self.sound.clone()
        };
        Arc::new(this)
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
            2 => {
                (self.samples[index * 2], self.samples[index * 2 + 1])
            }
            1 => {
                let sample = self.samples[index];
                (sample * 0.5, sample * 0.5)
            }
            _ => {
                panic!("We do not support futuristic 3+ ear headphones");
            }
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}