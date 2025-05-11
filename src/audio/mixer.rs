use std::sync::Arc;
use crate::audio::source::Sound;

pub struct AudioMixer { //gt reference
    playing: Vec<(Arc<Sound>, usize)>,
    last_idx: usize
}

impl AudioMixer {
    pub fn new() -> Self {
        Self {
            playing: vec![],
            last_idx: 0,
        }
    }

    pub fn play(&mut self, sound: Arc<Sound>) {
        self.playing.push((sound, self.last_idx));
    }

    pub(crate) fn get_current_sample(&mut self, idx: usize) -> f32 {
        self.last_idx = idx;
        let mut mixed = 0.0;
        let mut total = 0.0;
        for (i, (sound, started)) in self.playing.iter().enumerate() {
            if idx - started >= sound.samples.len() {
                continue;
            }
            let s = sound.samples[idx - started];
            mixed += s;
            total += 1.0;
        }
        
        self.playing.retain(|(sound, started)| idx - started < sound.samples.len());
        
        if total == 0.0 {
            return 0.0;
        }

        mixed / total
    }
}