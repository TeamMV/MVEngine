use crate::color::{Color, ColorFormat, HsvColor, RgbColor};
use mvutils::utils::Overlap;
use rand::Rng;

pub struct Colors;

impl Colors {
    /// generates one random color in the rgb color format
    pub fn random_rgb() -> RgbColor {
        let mut rng = rand::rng();

        let r = rng.random_range(0..255);
        let g = rng.random_range(0..255);
        let b = rng.random_range(0..255);
        RgbColor::new([r, g, b, 255])
    }

    /// generates one random color in the specified color format
    pub fn random<F: ColorFormat>() -> Color<F> {
        F::from_rgb(Self::random_rgb())
    }

    /// generates one random color in the hsv color format using a predefined lightness
    pub fn random_light(lightness: f32) -> HsvColor {
        let mut rng = rand::rng();
        let h = rng.random_range(0..359);
        HsvColor::new([h as f32, 1.0, lightness, 0.0])
    }

    /// generates <b>amt</b> random colors in the hsv color format using a predefined lightness. The colors are chosen so none is similar to another
    pub fn random_diff(lightness: f32, amt: u8) -> Vec<HsvColor> {
        let start = rand::rng().random_range(0..359);

        let step = 360.0 / amt as f32;

        let mut colors = vec![];
        for i in 0..amt {
            let h = ((i as f32 * step) as i32 + start).overlap(0, 359);
            colors.push(HsvColor::new([h as f32, 1.0, lightness, 0.0]))
        }
        colors
    }

    pub fn get_contrast<F1: ColorFormat, F2: ColorFormat, R: ColorFormat>(col: Color<F1>, pivot: Color<F2>) -> Color<R> {
        let start = col.to_hsv();
        let pivot = pivot.to_hsv();
        let new_hue = (2.0 * pivot.components[0] + start.components[0]).rem_euclid(360.0);
        let new_col = HsvColor::new([new_hue, start.components[1], start.components[2], start.components[3]]);
        R::from_rgb(new_col.to_rgb())
    }
}
