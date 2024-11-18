pub mod sound;

use std::collections::VecDeque;
use std::ffi::CString;
use std::io;
use std::io::ErrorKind;
use std::sync::Arc;
use al_sys::{ALCcontext, ALCdevice, ALuint, AlApi, ALC_DEFAULT_DEVICE_SPECIFIER, AL_NO_ERROR, AL_ORIENTATION, AL_POSITION, AL_SOURCE_STATE, AL_STOPPED, AL_VELOCITY};
use parking_lot::Mutex;
use crate::audio::sound::{Sound, SoundCreateInfo};

pub struct Audio {
    api: AlApi,

    device: *mut ALCdevice,
    context: *mut ALCcontext,

    max_sources: u32,
    sources: Mutex<Vec<ALuint>>,
    free_sources: Mutex<VecDeque<ALuint>>,
}

impl Audio {
    pub fn new(max_sources: u32) -> io::Result<Arc<Self>> {
        unsafe {
            let api = AlApi::load_default()?;
            let device_name = api.alcGetString(std::ptr::null_mut(), ALC_DEFAULT_DEVICE_SPECIFIER);
            let device = api.alcOpenDevice(device_name);
            if device.is_null() {
                let name = CString::from_raw(device_name as *mut _).to_string_lossy().to_string();
                return Err(io::Error::new(ErrorKind::NotFound, format!("Could not open audio device '{name}'")));
            }
            let context = api.alcCreateContext(device, &0);
            if context.is_null() {
                api.alcCloseDevice(device);
                let name = CString::from_raw(device_name as *mut _).to_string_lossy().to_string();
                return Err(io::Error::new(ErrorKind::PermissionDenied, format!("Could not create context for device '{name}'")));
            }
            if api.alcMakeContextCurrent(context) == 0 {
                api.alcCloseDevice(device);
                return Err(io::Error::new(ErrorKind::PermissionDenied, "Could not make context current"));
            }
            let mut sources = Vec::with_capacity(max_sources as usize);
            api.alGenSources(max_sources as i32, sources.as_mut_ptr());
            if api.alGetError() != AL_NO_ERROR {
                return Err(io::Error::new(ErrorKind::PermissionDenied, "Could not generate audio sources"));
            }

            let free_sources = Mutex::new(sources.iter().copied().collect());

            api.alListenerfv(AL_ORIENTATION, [0.0f32, 0.0, -1.0, 0.0, 1.0, 0.0].as_ptr());
            api.alListener3f(AL_VELOCITY, 0.0, 0.0, 0.0);
            api.alListener3f(AL_POSITION, 0.0, 0.0, 0.0);

            Ok(Audio {
                api,
                device,
                context,
                max_sources,
                sources: sources.into(),
                free_sources,
            }.into())
        }
    }

    pub fn create_sound(self: Arc<Self>, create_info: SoundCreateInfo) -> Sound {
        Sound::new(self, create_info)
    }

    pub(crate) fn next_source(&self) -> Option<ALuint> {
        self.free_sources.lock().pop_front()
    }

    pub(crate) fn free_source(&self, source: ALuint) {
        self.free_sources.lock().push_back(source);
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        /*

        alcDestroyContext(context);
        alcCloseDevice(device);
        instance = null;
         */
        unsafe {
            for source in self.sources.lock().iter() {
                let mut state = 0;
                self.api.alGetSourcei(*source, AL_SOURCE_STATE, &mut state);
                if state != AL_STOPPED {
                    self.api.alSourceStop(*source);
                }
            }
            self.api.alDeleteSources(self.max_sources as i32, self.sources.lock().as_mut_ptr());
            self.api.alcDestroyContext(self.context);
            self.api.alcCloseDevice(self.device);
        }
    }
}

unsafe impl Send for Audio {}
unsafe impl Sync for Audio {}