#![feature(new_uninit)]
#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]
// These are temporary during development, unused functions and variables will need to be
// used or removed before release
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_macros)]
#![allow(unused_assignments)]
//#![feature(specialization)]

use log::LevelFilter;
use std::sync::Arc;

use mvsync::{MVSync, MVSyncSpecs};
use mvutils::version::Version;

use crate::render::RenderCore;

mod err;
pub mod input;
mod parsing;
pub mod render;
pub mod resources;
#[cfg(feature = "ui")]
pub mod ui;
#[cfg(feature = "vr")]
pub mod vr;

pub use mvcore_proc_macro::ui_element;

pub struct MVCore {
    render: Arc<RenderCore>,
    sync: Arc<MVSync>,
    info: ApplicationInfo,
}

impl MVCore {
    pub fn new(info: ApplicationInfo) -> Arc<MVCore> {
        mvlogger::init(std::io::stdout(), LevelFilter::Off);
        //err::setup();
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
    pub fn new(
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
