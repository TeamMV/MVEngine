pub mod backend;
pub mod window;

pub trait ApplicationLoopCallbacks {
    fn start(&mut self);
    fn draw(&mut self);
    fn update(&mut self);
    fn end(&mut self);
}
