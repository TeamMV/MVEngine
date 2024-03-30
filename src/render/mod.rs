pub mod backend;
pub mod render2d;
pub mod window;

mod state;
mod camera2d;

pub trait ApplicationLoopCallbacks {
    fn start(&mut self);
    fn draw(&mut self);
    fn update(&mut self);
    fn end(&mut self);
}
