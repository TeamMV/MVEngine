pub mod backend;
pub mod window;
mod state;
mod render2d;
mod camera2d;

pub trait ApplicationLoopCallbacks {
    fn start(&mut self);
    fn draw(&mut self);
    fn update(&mut self);
    fn end(&mut self);
}
