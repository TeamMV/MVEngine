pub mod window;
pub(crate) mod init;
pub mod consts;
pub mod common;
pub(crate) mod render;
pub mod camera;
pub(crate) mod batch2d;
pub mod color;
pub mod draw;
pub mod text;

use std::sync::Arc;
use crate::render::window::{Window, WindowSpecs};

pub struct RenderCore;

impl RenderCore {
    pub fn new() -> Self {
        RenderCore
    }
   pub fn run_window(&self, info: WindowSpecs) {
       Window::run(info)
   }
}