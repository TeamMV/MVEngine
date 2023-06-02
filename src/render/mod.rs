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
#[cfg(feature = "3d")]
pub mod render3d;
#[cfg(feature = "3d")]
pub mod deferred;
#[cfg(feature = "3d")]
pub mod common3d;
#[cfg(feature = "3d")]
pub mod model;

use std::sync::Arc;
use crate::{ ApplicationLoopCallbacks};
use crate::render::window::{Window, WindowSpecs};

pub struct RenderCore;

impl RenderCore {
    pub fn new() -> Arc<Self> {
        Arc::new(RenderCore)
    }

    pub fn run_window<ApplicationLoop: ApplicationLoopCallbacks + Sized + 'static>(self: &Arc<RenderCore>, info: WindowSpecs, application_loop: ApplicationLoop) {
        Window::run(info, self.clone(), application_loop)
    }
}