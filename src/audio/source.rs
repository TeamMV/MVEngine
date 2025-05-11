use std::sync::Arc;

pub struct Sound {
    pub samples: Vec<f32>,
}

impl Sound {
    pub fn new(samples: Vec<f32>) -> Arc<Self> {
        let this = Self {
            samples
        };
        Arc::new(this)
    }
}