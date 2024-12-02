use ash::vk::SECURITY_ATTRIBUTES;
use windows::Win32::Foundation::{BOOL, HANDLE};
use windows::Win32::Media::Audio::{eConsole, eRender, IAudioClient, IAudioClient2, IAudioRenderClient, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator, AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM, AUDCLNT_STREAMFLAGS_EVENTCALLBACK, AUDCLNT_STREAMFLAGS_RATEADJUST, AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY, WAVEFORMATEX, WAVE_FORMAT_PCM};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED, COINIT_SPEED_OVER_MEMORY};
use windows::Win32::System::Threading::{CreateEventA};

pub struct WindowsAudio {
    device: IMMDevice,
    audio_client: IAudioClient2,
    render_client: IAudioRenderClient
}

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

            let buffer_duration = 20000000;
            audio_client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_RATEADJUST
                    | AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM
                    | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY,
                buffer_duration,
                0,
                (&wave_format as *const WAVEFORMATEX),
                None,
            )?;

            //throws some sigsegv error
            //CoTaskMemFree(Some(&mut wave_format as *mut WAVEFORMATEX as *mut _));

            let render_client: IAudioRenderClient = audio_client.GetService()?;

            Ok(Self {
                device,
                audio_client,
                render_client,
            })
        }
    }

    pub fn play_loop(&self, wave_data: Vec<i16>, sample_rate: u32) -> windows::core::Result<()> {
        unsafe {
            let buffer_size_in_frames = self.audio_client.GetBufferSize()?; // Total buffer size
            let mut wav_playback_sample = 0; // Playback position in wave data
            let num_wav_samples = wave_data.len();

            self.audio_client.Start()?;

            loop {
                let buffer_padding = self.audio_client.GetCurrentPadding()?;

                const TARGET_BUFFER_PADDING_IN_SECONDS: f32 = 1.0 / 60.0;
                let target_buffer_padding = (buffer_size_in_frames as f32 * TARGET_BUFFER_PADDING_IN_SECONDS) as u32;

                let num_frames_to_write = if target_buffer_padding > buffer_padding {
                    target_buffer_padding - buffer_padding
                } else {
                    0
                };

                if num_frames_to_write > 0 {
                    let buffer = self
                        .render_client
                        .GetBuffer(num_frames_to_write)? as *mut i16;


                    for frame_index in 0..num_frames_to_write {
                        let left_sample = wave_data[wav_playback_sample];
                        let right_sample = wave_data[wav_playback_sample];
                        wav_playback_sample += 1;

                        std::ptr::write(buffer.add(frame_index as usize * 2), left_sample);
                        std::ptr::write(buffer.add(frame_index as usize * 2 + 1), right_sample);

                        if wav_playback_sample >= num_wav_samples {
                            wav_playback_sample = 0;
                        }
                    }

                    self.render_client.ReleaseBuffer(num_frames_to_write, 0)?;

                    println!("6");
                }

                //idk chatgpt put this there to not "hog the cpu"
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }
}

pub struct WindowsSound {
    data: Vec<u8>
}