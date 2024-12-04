use std::sync::Arc;
use ash::vk::SECURITY_ATTRIBUTES;
use log::{debug, warn};
use parking_lot::Mutex;
use windows::core::imp::WaitForSingleObject;
use windows::Win32::Foundation::{BOOL, HANDLE};
use windows::Win32::Media::Audio::{eConsole, eRender, IAudioClient, IAudioClient2, IAudioRenderClient, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator, AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM, AUDCLNT_STREAMFLAGS_EVENTCALLBACK, AUDCLNT_STREAMFLAGS_RATEADJUST, AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY, WAVEFORMATEX, WAVE_FORMAT_PCM};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED, COINIT_SPEED_OVER_MEMORY};
use windows::Win32::System::Threading::{CreateEventA};
use crate::audio::AudioImpl;

pub struct WindowsAudio {
    device: IMMDevice,
    audio_client: Arc<Mutex<IAudioClient2>>,
    render_client: Arc<Mutex<IAudioRenderClient>>,
    event_handle: HANDLE
}

unsafe impl Send for WindowsAudio {}

impl WindowsAudio {
    pub fn setup() -> windows::core::Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_SPEED_OVER_MEMORY).ok()?;

            let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let device: IMMDevice = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

            let audio_client: IAudioClient2 = device.Activate(CLSCTX_ALL, None)?;

            let mut wave_format = WAVEFORMATEX::default();
            wave_format.wFormatTag = WAVE_FORMAT_PCM as u16;
            wave_format.nChannels = 2;
            wave_format.nSamplesPerSec = 44100;
            wave_format.wBitsPerSample = 16;
            wave_format.nBlockAlign = (wave_format.nChannels * wave_format.wBitsPerSample) / 8;
            wave_format.nAvgBytesPerSec = wave_format.nSamplesPerSec * wave_format.nBlockAlign as u32;

            let buffer_duration = 10000000;
            audio_client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                    AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM
                    | AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
                buffer_duration,
                0,
                (&wave_format as *const WAVEFORMATEX),
                None,
            )?;

            let event = CreateEventA(None, false, false, None)?;

            audio_client.SetEventHandle(event)?;

            let render_client: IAudioRenderClient = audio_client.GetService()?;

            audio_client.Start()?;

            Ok(Self {
                device,
                audio_client: Arc::new(Mutex::new(audio_client)),
                render_client: Arc::new(Mutex::new(render_client)),
                event_handle: event,
            })
        }
    }

    pub(crate) unsafe fn write_samples(&mut self, samples: &[i16]) {
        let lock = self.audio_client.lock();
        let buffer_size_in_frames = lock.GetBufferSize().unwrap();
        let buffer_padding = lock.GetCurrentPadding().unwrap();


        let available_frames = buffer_size_in_frames - buffer_padding;
        let frames_to_write = samples.len() / 2;
        let num_frames_to_write = frames_to_write.min(available_frames as usize);

        if num_frames_to_write <= 0 {
            debug!("Skipping audio frames...");
        }


        let buffer = match self.render_client.lock().GetBuffer(num_frames_to_write as u32) {
            Ok(buffer) => buffer,
            Err(e) => {
                warn!("Failed to get buffer: {}", e);
                return;
            }
        } as *mut i16;


        unsafe {
            for (i, sample) in samples.iter().take(num_frames_to_write * 2).enumerate() {
                std::ptr::write(buffer.add(i), *sample);
            }
        }

        self.render_client.lock().ReleaseBuffer(num_frames_to_write as u32, 0).unwrap();

        while WaitForSingleObject(self.event_handle.0, 0xFFFFFFFF) != 0 {
            println!("waiting...");
        }
    }
}