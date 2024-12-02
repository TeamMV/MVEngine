pub mod sound;
pub mod backend;

pub fn adjust_volume(data: Vec<i16>, fac: f32) -> Vec<i16> {
    data.into_iter()
        .map(|sample| (sample as f32 * fac) as i16)
        .collect()
}

pub fn adjust_pan(data: Vec<i16>, pan: f32) -> Vec<i16> {
    let mut adjusted_data = Vec::new();
    for i in (0..data.len()).step_by(2) {
        let left_sample = data[i];
        let right_sample = if i + 1 < data.len() {
            data[i + 1]
        } else {
            0
        };

        let left_adjusted = (left_sample as f32 * (1.0 - pan)).round() as i16;
        let right_adjusted = (right_sample as f32 * (1.0 + pan)).round() as i16;

        adjusted_data.push(left_adjusted);
        adjusted_data.push(right_adjusted);
    }
    adjusted_data
}