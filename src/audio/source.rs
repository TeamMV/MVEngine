use std::mem;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use crate::audio::decode::wav::ActuallyUsefulWavData;

pub struct Sound {
    // hacky ass workaround, use raw bytes lmao
    mono_balance: AtomicU32,
    looping: AtomicBool,
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
            mono_balance: AtomicU32::new(unsafe { mem::transmute(0.5f32) }),
            looping: AtomicBool::new(false),
            channels,
            sample_rate,
            samples,
        };
        Arc::new(this)
    }

    pub fn get_sample(&self, index: usize) -> (f32, f32) {
        if self.samples.is_empty() {
            return (0.0, 0.0);
        }
        
        let looping = self.looping.load(Ordering::Acquire);
        
        match self.channels {
            2 => {
                let sample_count = self.samples.len() / 2;
                let idx = if looping {
                    index % sample_count
                } else {
                    index
                };
                (self.samples[idx * 2], self.samples[idx * 2 + 1])
            }
            1 => {
                let balance = unsafe { mem::transmute::<u32, f32>(self.mono_balance.load(Ordering::Acquire)) };
                let sample_count = self.samples.len();
                let idx = if looping {
                    index % sample_count
                } else {
                    index
                };
                let sample = self.samples[idx];
                (sample * (1.0 - balance), sample * balance)
            }
            _ => {
                panic!("We do not support futuristic 3+ ear headphones");
            }
        }
    }
    
    pub fn is_looping(&self) -> bool {
        self.looping.load(Ordering::Acquire)
    }
    
    pub fn set_looping(&mut self, looping: bool) {
        self.looping.store(looping, Ordering::Release);
    }
    
    pub fn balance(&self) -> f32 {
        unsafe { mem::transmute(self.mono_balance.load(Ordering::Acquire)) }
    }
    
    pub fn set_balance(&mut self, balance: f32) {
        self.mono_balance.store(unsafe { mem::transmute(balance) }, Ordering::Release);
    }
    
    pub fn total_samples(&self) -> usize {
        self.samples.len()
    }
}