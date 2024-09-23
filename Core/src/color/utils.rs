use mvutils::utils::Overlap;
use rand::Rng;
use crate::color::{Color, ColorFormat, HsvColor, RgbColor};

pub struct C;

impl C {
    /// generates one random color in the rgb color format
    pub fn random_rgb() -> RgbColor {
        let mut rng = rand::thread_rng();

        let r = rng.gen_range(0..255);
        let g = rng.gen_range(0..255);
        let b = rng.gen_range(0..255);
        RgbColor::new([r, g, b, 255])
    }

    /// generates one random color in the specified color format
    pub fn random<F: ColorFormat>() -> Color<F> {
        F::from_rgb(Self::random_rgb())
    }

    /// generates one random color in the hsv color format using a predefined lightness
    pub fn random_light(lightness: f32) -> HsvColor {
        let mut rng = rand::thread_rng();
        let h = rng.gen_range(0..359);
        HsvColor::new([h as f32, 1.0, lightness, 0.0])
    }

    /// generates <b>amt</b> random colors in the hsv color format using a predefined lightness. The colors are chosen so none is similar to another
    pub fn random_diff(lightness: f32, amt: u8) -> Vec<HsvColor> {
        let start = rand::thread_rng().gen_range(0..359);

        let step = 360.0 / amt as f32;

        let mut colors = vec![];
        for i in 0.0..amt as f32 {
            let h = ((i * step) as i32 + start).overlap(0, 359);
            colors.push(HsvColor::new([h as f32, 1.0, lightness, 0.0]))
        }
        colors
    }
}