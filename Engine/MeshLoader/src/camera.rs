pub enum Camera {
    P(PCamera),
    O(OCamera)
}

pub struct PCamera {
    pub aspect: f64,
    pub fov: f64,
    pub z_far: Option<f64>,
    pub z_near: f64
}

pub struct OCamera {
    pub x_mag: f64,
    pub y_mag: f64,
    pub z_far: Option<f64>,
    pub z_near: f64
}