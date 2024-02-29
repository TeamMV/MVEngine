mod background;
pub mod ease;
pub mod elements;
pub mod parsing;
pub mod prelude;
pub mod styles;
pub mod timing;

pub struct Sides {
    pub top: i32,
    pub bottom: i32,
    pub left: i32,
    pub right: i32,
}

impl Sides {
    pub fn copy_slice(&mut self, data: &[i32]) {
        self.top = data[0];
        self.bottom = data[1];
        self.left = data[2];
        self.right = data[3];
    }

    pub fn same(val: i32) -> Self {
        Self {
            top: val,
            bottom: val,
            left: val,
            right: val,
        }
    }
}
