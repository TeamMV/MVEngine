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
use mvutils::unsafe_utils::Nullable;
use crate::MVCore;
use crate::render::window::{Window, WindowSpecs};

pub struct RenderCore {
    core: Nullable<Arc<MVCore>>
}

impl RenderCore {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(RenderCore {
            core: Nullable::null()
        })
    }

    pub(crate) fn set_core(&self, core: Arc<MVCore>) {
        let this = unsafe { &mut *(self as *const RenderCore).cast_mut() };
        this.core = Nullable::new(core);
    }

    pub fn run_window<ApplicationLoop: ApplicationLoopCallbacks + Sized + 'static>(self: &Arc<RenderCore>, info: WindowSpecs, application_loop: ApplicationLoop) {
        Window::run(info, application_loop)
    }
}

pub trait ApplicationLoopCallbacks: Sized {
    fn start(&self, window: Arc<Window<Self>>);
    fn update(&self, window: Arc<Window<Self>>);
    fn draw(&self, window: Arc<Window<Self>>);
    fn exit(&self, window: Arc<Window<Self>>);
}