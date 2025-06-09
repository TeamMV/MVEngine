pub trait MapTo {
    fn map_to(self, amount: u32) -> u32;
}

impl MapTo for f64 {
    fn map_to(self, amount: u32) -> u32 {
        (self * amount as f64) as u32
    }
}
