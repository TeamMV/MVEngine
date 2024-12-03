use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::Duration;
use mvutils::unsafe_utils::Unsafe;
use crate::audio::backend::windows::WindowsAudio;
use crate::audio::sound::{Hearable, SimpleSound};
use crate::math::vec::Vec3;

pub mod sound;
pub mod backend;

pub fn adjust_volume(data: Vec<i16>, fac: f32) -> Vec<i16> {
    data.into_iter()
        .map(|sample| (sample as f32 * fac) as i16)
        .collect()
}

pub fn adjust_pan(data: Vec<i16>, pan: f32) -> Vec<i16> {
    let mut adjusted_data = Vec::new();
    for i in (0..data.len()).step_by(2) {
        let left_sample = data[i];
        let right_sample = if i + 1 < data.len() {
            data[i + 1]
        } else {
            0
        };

        let left_adjusted = (left_sample as f32 * (1.0 - pan)).round() as i16;
        let right_adjusted = (right_sample as f32 * (1.0 + pan)).round() as i16;

        adjusted_data.push(left_adjusted);
        adjusted_data.push(right_adjusted);
    }
    adjusted_data
}

enum AudioImpl {
    NoOs,
    Windows(WindowsAudio),
    Linux(()),
    MacOs(())
}

unsafe impl Send for AudioImpl {}

pub struct Audio {
    implementation: AudioImpl,
    listener: Vec3,
    current_sources: usize,
    audio_sources: usize,
    mixer: Mixer,
    started: bool,
    alive: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>
}

unsafe impl Send for Audio {}

impl Audio {
    pub fn new(audio_sources: usize) -> Self {
        let im = AudioImpl::Windows(WindowsAudio::setup().expect("Failed to initialize windows audio!"));

        Self {
            implementation: im,
            listener: Vec3::default(),
            current_sources: 0,
            audio_sources,
            mixer: Mixer::new(44100, 2, 1024),
            started: false,
            alive: Arc::new(AtomicBool::new(true)),
            thread: None,
        }
    }

    pub fn start(&mut self) {
        unsafe {
            if !self.started {
                self.started = true;

                let this = Unsafe::cast_mut_static(self);

                let thread = thread::spawn(|| unsafe {
                    while this.alive.load(Ordering::Acquire) {
                        let mixed = this.mixer.mix();
                        match &mut this.implementation {
                            AudioImpl::NoOs => unreachable!(),
                            AudioImpl::Windows(win) => win.write_samples(mixed),
                            AudioImpl::Linux(_) => {}
                            AudioImpl::MacOs(_) => {}
                        }

                        sleep(Duration::from_millis(10));
                    }
                });

                self.thread = Some(thread)
            }
        }
    }

    pub fn stop(&mut self) {
        self.alive.store(false, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }

    pub fn play(&mut self, sound: Hearable, queue: bool) {
        if queue && self.current_sources >= self.audio_sources {
            //queue sound
            return;
        }

        self.mixer.play_sound(sound);
    }
}

struct PlayingSound(Hearable, usize);

impl PlayingSound {
    fn new(sound: Hearable) -> Self {
        Self(sound, 0)
    }
}

struct Mixer {
    playing: Vec<PlayingSound>,
    mixed: Vec<i16>,
    sample_rate: u32,
    channels: u16
}

impl Mixer {
    pub fn new(sample_rate: u32, channels: u16, buffer_size: usize) -> Self {
        Self {
            playing: Vec::new(),
            mixed: vec![0; buffer_size * channels as usize],
            sample_rate,
            channels,
        }
    }

    fn play_sound(&mut self, sound: Hearable) {
        self.playing.push(PlayingSound::new(sound))
    }

    fn mix(&mut self) -> &[i16] {
        self.mixed.fill(0);

        for playing in &mut self.playing {
            let sound = &playing.0;
            let mut position = playing.1;
            for i in 0..self.mixed.len() / 2 {
                if position >= sound.length() {
                    continue;
                }

                let sample = sound.data()[position];
                self.mixed[i * 2] = self.mixed[i * 2].saturating_add(sample);
                self.mixed[i * 2 + 1] = self.mixed[i * 2 + 1].saturating_add(sample);
                position += 1;
                playing.1 = position;
            }
        }

        self.playing.retain(|pl| pl.1 < pl.0.length());

        &self.mixed
    }
}