use std::f32::consts::PI;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use log::LevelFilter;
use kiss3d::camera::{ArcBall, FirstPerson};
use kiss3d::event::{Action, Key, WindowEvent};
use kiss3d::light::Light;
use kiss3d::window::Window;
use mvutils::unsafe_utils::DangerousCell;
use nalgebra::geometry::Point3;
use mvcore::audio::backend::windows::WindowsAudio;
use mvcore::audio::{adjust_pan, adjust_volume, Audio};
use mvcore::audio::sound::{Hearable, SimpleSound};
use mvcore::math::vec::Vec3;

fn generate_sine_wave(sample_rate: u32, frequency: f32, duration: f32) -> Vec<i16> {
    let num_samples = (duration * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let time = i as f32 / sample_rate as f32;
        let sample = (f32::sin(2.0 * PI * frequency * time) * i16::MAX as f32) as i16;
        samples.push(sample);
    }

    samples
}

fn main() {
    mvlogger::init(std::io::stdout(), LevelFilter::Info);
    mvcore::err::setup();

    let mut wav = hound::WavReader::open("vent_louder2.wav").expect("File could not be resolved!");

    let mut samples = Vec::new();
    for sample in wav.samples::<i16>() {
        samples.push(sample.map_err(|e| format!("Error reading sample: {}", e)).unwrap());
    }

    let samples = adjust_volume(samples, 0.8);
    let samples = adjust_pan(samples, 0.0);

    let sample_rate = 44100;
    let frequency = 440.0; // Frequency of A4 (440 Hz)
    let duration = 1.0; // 1 second of audio
    let sine_samples = generate_sine_wave(sample_rate, frequency, duration);

    let sine_samples = adjust_volume(sine_samples, 0.5);

    let sound: Arc<Hearable> = Arc::new(SimpleSound::new(samples, wav.spec().sample_rate).to_3d(Vec3::splat(1.0)).into());
    let sine: Arc<Hearable> = Arc::new(SimpleSound::new(sine_samples, sample_rate).to_3d(Vec3::splat(1.0)).into());

    let mut audio = Audio::new(2);
    audio.start();



    let block = Arc::new(AtomicBool::new(true));
    let cloned_block = block.clone();

    thread::spawn(move || {
        let eye = Point3::new(10.0f32, 10.0, 10.0);
        let at = Point3::origin();
        let mut first_person = FirstPerson::new(eye, at);
        first_person.rebind_down_key(Some(Key::S));
        first_person.rebind_up_key(Some(Key::W));
        first_person.rebind_left_key(Some(Key::A));
        first_person.rebind_right_key(Some(Key::D));

        let mut window = Window::new("Kiss3d: camera");
        window.set_light(Light::StickToCamera);

        let mut c = window.add_cube(1.0, 1.0, 1.0);
        c.set_color(1.0, 0.0, 0.0);


        while !window.should_close() {

            // update the current camera.
            for event in window.events().iter() {
                match event.value {
                    _ => {}
                }
            }

            window.render_with_camera(&mut first_person);
        }
        block.store(false, Ordering::Release);
    });

    audio.play(sound.clone(), false);
    //audio.play(sine.clone(), false);
    while cloned_block.load(Ordering::Acquire) {
        sleep(Duration::from_secs(1));
    }

    audio.stop();
}