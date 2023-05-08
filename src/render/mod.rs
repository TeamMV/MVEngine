pub mod window;
pub(crate) mod init;
pub mod consts;
pub mod common;
pub(crate) mod render;

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