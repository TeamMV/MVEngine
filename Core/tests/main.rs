use std::fs::OpenOptions;
use log::LevelFilter;
use mvcore::audio::Audio;
use mvcore::audio::sound::SoundCreateInfo;
use mvcore::math::vec::Vec3;

fn main() {
    mvlogger::init(std::io::stdout(), LevelFilter::Debug);
    mvcore::err::setup();

    let audio = Audio::new(32).unwrap();
    let file = OpenOptions::new().read(true).open("danger2b.wav").unwrap();
    let reader = hound::WavReader::new(file).unwrap();
    let spec = reader.spec();
    let data = reader.into_samples::<i16>().fold(vec![], |mut acc, sample| {
        if let Ok(sample) = sample {
            let bytes = sample.to_be_bytes();
            acc.push(bytes[0]);
            acc.push(bytes[1]);
        }
        acc
    });
    let mut sound = audio.create_sound(SoundCreateInfo {
        data,
        channels: spec.channels as u32,
        sample_rate: spec.sample_rate,
        looped: false,
        volume: 10.0,
        position: Vec3::new(1.0, 0.0, 1.0),
    });
    sound.play();
    
    loop {}
}
