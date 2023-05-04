pub mod window;
pub(crate) mod init;

use std::sync::Arc;
use crate::render::window::{Window, WindowSpecs};

pub struct RenderCore;

impl RenderCore {
    pub fn new() -> Self {
        RenderCore
    }

    pub fn create_window(&self, info: WindowSpecs) -> Window {
        Window::new(info)
    }
}