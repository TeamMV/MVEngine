use crate::math::vec::Vec3;

pub struct SimpleSound {
    wav_data: Vec<i16>,
    sample_rate: u32
}

impl SimpleSound {
    pub fn new(data: Vec<i16>, sample_rate: u32) -> Self {
        Self {
            wav_data: data,
            sample_rate,
        }
    }
}

pub struct ThreeDeeSound {
    sound: SimpleSound,
    pos: Vec3
}

impl ThreeDeeSound {
    pub fn new(sound: SimpleSound, pos: Vec3) -> Self {
        Self { sound, pos }
    }
}

pub enum Hearable {
    Simple(SimpleSound),
    ThreeDee(ThreeDeeSound)
}

impl Hearable {
    pub fn length(&self) -> usize {
        match self {
            Hearable::Simple(s) => s.wav_data.len(),
            Hearable::ThreeDee(s) => s.sound.wav_data.len()
        }
    }

    pub fn data(&self) -> &[i16] {
        match self {
            Hearable::Simple(s) => &s.wav_data,
            Hearable::ThreeDee(s) => &s.sound.wav_data //implement pan and volume logic
        }
    }
}

impl From<SimpleSound> for Hearable {
    fn from(value: SimpleSound) -> Self {
        Hearable::Simple(value)
    }
}

impl From<ThreeDeeSound> for Hearable {
    fn from(value: ThreeDeeSound) -> Self {
        Hearable::ThreeDee(value)
    }
}