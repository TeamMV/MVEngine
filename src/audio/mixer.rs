use crate::audio::source::SoundWithAttributes;
use std::sync::Arc;

pub struct AudioMixer {
    //gt reference
    playing: Vec<(Arc<SoundWithAttributes>, usize)>,
    last_idx: usize,
}

impl AudioMixer {
    pub fn new() -> Self {
        Self {
            playing: vec![],
            last_idx: 0,
        }
    }

    pub fn play(&mut self, sound: Arc<SoundWithAttributes>) {
        self.playing.push((sound, self.last_idx));
    }

    pub(crate) fn get_current_sample(&mut self, idx: usize, sample_rate: u32) -> (f32, f32) {
        self.last_idx = idx;
        let mut mixed = (0.0, 0.0);
        for (sound, started) in self.playing.iter() {
            let s = sound.get_sample_mapped(idx - started, sample_rate);
            mixed.0 += s.0;
            mixed.1 += s.1;
        }

        self.playing.retain(|(sound, started)| {
            sound.is_looping() || idx - started < sound.sound().total_samples()
        });

        // This feels like adding empty sound would just make shit quieter randomly, perhaps just use clamp and sum instead?
        // (clamp(mixed.0, -1.0, 1.0), clamp(mixed.1, -1.0, 1.0))
        mixed
    }
}
