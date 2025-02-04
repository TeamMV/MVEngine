use crate::window::{UninitializedWindow, Window};

pub trait WindowCallbacks {
    fn new(window: UninitializedWindow) -> Self where Self: Sized;
    fn post_init(&mut self, window: &mut Window);
    fn update(&mut self, window: &mut Window, delta_u: f64);
    fn draw(&mut self, window: &mut Window, delta_t: f64);
    fn exiting(&mut self, window: &mut Window);
    fn resize(&mut self, window: &mut Window, width: u32, height: u32);
}
