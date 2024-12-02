use std::f32::consts::PI;
use log::LevelFilter;
use mvcore::audio::backend::windows::WindowsAudio;
use mvcore::audio::{adjust_pan, adjust_volume};

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
    mvlogger::init(std::io::stdout(), LevelFilter::Debug);
    mvcore::err::setup();

    let mut wav = hound::WavReader::open("vent_louder2.wav").expect("File could not be resolved!");

    let mut samples = Vec::new();
    for sample in wav.samples::<i16>() {
        samples.push(sample.map_err(|e| format!("Error reading sample: {}", e)).unwrap());
    }

    let samples = adjust_volume(samples, 0.8);
    let samples = adjust_pan(samples, 0.0);

    //let sample_rate = 44100;
    //let frequency = 440.0; // Frequency of A4 (440 Hz)
    //let duration = 1.0; // 1 second of audio
    //let samples = generate_sine_wave(sample_rate, frequency, duration);
//
    //let samples = adjust_volume(samples, 0.5);

    let mut audio = WindowsAudio::setup();
    if let Err(e) = audio {
        print!("Error: {e}");
        return;
    }

    let audio = audio.unwrap();

    println!("init done");
    if let Err(e) = audio.play_loop(samples, wav.spec().sample_rate) {
        println!("Error: {e}");
    }
}
