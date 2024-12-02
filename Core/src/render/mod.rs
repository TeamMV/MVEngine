use crate::render::window::Window;

pub mod backend;
pub mod mesh;
pub mod renderer;
pub mod window;

pub mod camera;
pub mod model;
pub mod texture;

pub trait ApplicationLoopCallbacks {
    fn new(window: &mut Window) -> Self;
    fn post_init(&mut self, window: &mut Window);
    fn update(&mut self, window: &mut Window, delta_t: f64);
    fn draw(&mut self, window: &mut Window, delta_t: f64);
    fn exiting(&mut self, window: &mut Window);
    fn resize(&mut self, window: &mut Window, width: u32, height: u32);
}
