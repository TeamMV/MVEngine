use std::sync::Arc;

use mvutils::unsafe_utils::Nullable;

use crate::render::window::{Window, WindowSpecs};
use crate::MVCore;

pub(crate) mod batch2d;
#[cfg(feature = "3d")]
pub(crate) mod batch3d;
pub mod camera;
pub mod color;
pub mod common;
#[cfg(feature = "3d")]
pub mod common3d;
pub mod consts;
#[cfg(feature = "3d")]
pub mod deferred;
pub mod draw2d;
#[cfg(feature = "3d")]
pub mod draw3d;
pub(crate) mod init;
#[cfg(feature = "3d")]
pub mod model;
pub(crate) mod render;
#[cfg(feature = "3d")]
pub mod render3d;
pub mod text;
pub mod window;

pub struct RenderCore {
    core: Nullable<Arc<MVCore>>,
}

impl RenderCore {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(RenderCore {
            core: Nullable::null(),
        })
    }

    pub(crate) fn set_core(&self, core: Arc<MVCore>) {
        let this = unsafe { &mut *(self as *const RenderCore).cast_mut() };
        this.core = Nullable::new(core);
    }

    pub fn run_window<ApplicationLoop: ApplicationLoopCallbacks + Sized + 'static>(
        self: &Arc<RenderCore>,
        info: WindowSpecs,
        application_loop: ApplicationLoop,
    ) {
        Window::run(info, application_loop)
    }
}

unsafe impl Send for RenderCore {}

unsafe impl Sync for RenderCore {}

pub trait ApplicationLoopCallbacks: Sized {
    fn start(&self, window: Arc<Window<Self>>);
    fn update(&self, window: Arc<Window<Self>>);
    fn draw(&self, window: Arc<Window<Self>>);
    fn exit(&self, window: Arc<Window<Self>>);
}
