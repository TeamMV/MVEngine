use std::sync::Arc;
use al_sys::{ALuint, AL_BUFFER, AL_FORMAT_MONO16, AL_FORMAT_STEREO16, AL_GAIN, AL_LOOPING, AL_PAUSED, AL_PLAYING, AL_POSITION, AL_SOURCE_STATE, AL_STOPPED};
use crate::audio::Audio;
use crate::math::vec::Vec3;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SoundState {
    Stopped,
    Paused,
    Playing,
}

pub struct Sound {
    audio: Arc<Audio>,
    looped: bool,
    state: SoundState,
    volume: f32,
    position: Vec3,
    buffer: u32,
    id: Option<ALuint>,
}

impl Sound {
    pub(crate) fn new(audio: Arc<Audio>, create_info: SoundCreateInfo) -> Self {
        unsafe {
            let mut buffer = 0;
            audio.api.alGenBuffers(1, &mut buffer);
            audio.api.alBufferData(
                buffer,
                if create_info.channels > 1 { AL_FORMAT_STEREO16 } else { AL_FORMAT_MONO16 },
                create_info.data.as_ptr() as *const _,
                create_info.data.len() as _,
                create_info.sample_rate as _,
            );
            Sound {
                audio,
                looped: create_info.looped,
                state: SoundState::Stopped,
                volume: create_info.volume,
                position: create_info.position,
                buffer,
                id: None,
            }
        }
    }

    pub fn get_state(&self) -> SoundState {
        self.state
    }

    pub fn play(&mut self) {
        unsafe {
            self.update_state();
            let id = if let Some(id) = self.audio.next_source() { id } else { return; };
            self.id = Some(id);
            self.audio.api.alSourcei(id, AL_BUFFER, self.buffer as _);
            self.audio.api.alSourcei(id, AL_LOOPING, if self.looped { 0 } else { 1 });
            self.audio.api.alSourcef(id, AL_GAIN, self.volume);
            self.update_position();
            if self.state != SoundState::Playing {
                self.state = SoundState::Playing;
                self.audio.api.alSourcePlay(id);
            }
        }
    }

    pub fn stop(&mut self) {
        unsafe {
            if let Some(id) = self.id {
                self.update_state();
                if self.state != SoundState::Stopped {
                    self.audio.api.alSourceStop(id);
                    self.audio.api.alSourcei(id, AL_BUFFER, 0);
                }
                self.id = None;
                self.audio.free_source(id);
                self.state = SoundState::Stopped;
            }
        }
    }
    
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        if self.state != SoundState::Stopped {
            unsafe { self.update_position(); }
        }
    }

    unsafe fn update_state(&mut self) {
        if let Some(id) = self.id {
            let mut state = 0;
            self.audio.api.alGetSourcei(id, AL_SOURCE_STATE, &mut state);
            self.state = match state {
                AL_PLAYING => SoundState::Playing,
                AL_PAUSED => SoundState::Paused,
                AL_STOPPED => SoundState::Stopped,
                _ => SoundState::Stopped,
            };
        } else {
            self.state = SoundState::Stopped;
        }
    }

    unsafe fn update_position(&mut self) {
        if let Some(id) = self.id {
            self.audio.api.alSource3f(id, AL_POSITION, self.position.x, self.position.y, self.position.z);
        }
    }
}

impl Drop for Sound {
    fn drop(&mut self) {
        unsafe {
            if self.state != SoundState::Stopped {
                self.stop();
            }
            self.audio.api.alDeleteBuffers(1, &self.buffer);
        }
    }
}

pub struct SoundCreateInfo {
    pub data: Vec<u8>,
    pub channels: u32,
    pub sample_rate: u32,
    pub looped: bool,
    pub volume: f32,
    pub position: Vec3,
}