extern crate alloc;
extern crate core;

use std::sync::Arc;

use mvsync::{MVSync, MVSyncSpecs};
use mvutils::version::Version;

use crate::render::RenderCore;

#[cfg(feature = "gui")]
pub mod gui;
pub mod input;
pub mod render;
pub mod resources;

pub struct MVCore {
    render: Arc<RenderCore>,
    sync: Arc<MVSync>,
    info: ApplicationInfo,
}

impl MVCore {
    pub fn new(info: ApplicationInfo) -> Arc<MVCore> {
        let core = if info.multithreaded {
            MVCore {
                render: RenderCore::new(),
                sync: MVSync::labelled(
                    MVSyncSpecs {
                        thread_count: info.extra_threads + 1,
                        workers_per_thread: 16,
                    },
                    vec!["update"],
                ),
                info,
            }
        } else {
            MVCore {
                render: RenderCore::new(),
                sync: MVSync::new(MVSyncSpecs {
                    thread_count: info.extra_threads,
                    workers_per_thread: 16,
                }),
                info,
            }
        };
        let core = Arc::new(core);
        core.render.set_core(core.clone());
        core
    }

    //pub fn loading_screen(self: &Arc<MVCore>, specs: LoadingScreenSpecs) -> Arc<LoadingScreen> {
    //    LoadingScreen::new(self.clone(), specs)
    //}

    pub fn get_app_version(self: &Arc<MVCore>) -> Version {
        self.info.version
    }

    pub fn get_render(self: &Arc<MVCore>) -> Arc<RenderCore> {
        self.render.clone()
    }

    pub fn get_sync(self: &Arc<MVCore>) -> Arc<MVSync> {
        self.sync.clone()
    }
}

impl Drop for MVCore {
    fn drop(&mut self) {}
}

unsafe impl Send for MVCore {}

unsafe impl Sync for MVCore {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ApplicationInfo {
    pub name: String,
    pub version: Version,
    pub multithreaded: bool,
    pub extra_threads: u32,
}

impl Default for ApplicationInfo {
    fn default() -> Self {
        ApplicationInfo {
            name: "MVCore application".to_string(),
            version: Version::default(),
            multithreaded: true,
            extra_threads: 1,
        }
    }
}

impl ApplicationInfo {
    fn new(
        name: &str,
        version: Version,
        multithreaded: bool,
        extra_threads: u32,
    ) -> ApplicationInfo {
        ApplicationInfo {
            name: name.to_string(),
            version,
            multithreaded,
            extra_threads,
        }
    }
}
