extern crate alloc;
extern crate core;

use std::cell::{Ref, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender};
use std::thread::{JoinHandle, spawn};

use include_dir::{Dir, include_dir};
use mvutils::version::Version;

//use crate::assets::{AssetManager, ReadableAssetManager, SemiAutomaticAssetManager};
use crate::render::RenderCore;
use crate::render::window::WindowSpecs;
//use crate::resource_loader::{AssetManager, LoadRequest, ResourceLoader};

//pub mod assets;
pub mod input;
//pub mod resource_loader;
pub mod files;
//#[cfg(feature = "gui")]
//pub mod gui;
pub mod render;

pub struct MVCore {
    render: Arc<RenderCore>,
    //load_request: Sender<LoadRequest>,
    resource_thread: JoinHandle<()>,
    //resource_loader: Arc<ResourceLoader>,
    info: ApplicationInfo
}

impl MVCore {
    pub fn new(info: ApplicationInfo) -> MVCore {
        static DIR: Dir = include_dir!("assets");
        //let mut assets = AssetManager::semi_automatic(DIR.clone());
        todo!()
    }

    pub fn get_app_version(&self) -> Version {
        self.info.version
    }

    pub fn get_render(&self) -> Arc<RenderCore> {
        self.render.clone()
    }

    //pub fn get_asset_manager(&self) -> &dyn ReadableAssetManager {
    //    self.resource_loader.get_asset_manager("MVCore")
    //}

    //pub fn get_resource_loader(&self) -> Arc<ResourceLoader> {
    //    self.resource_loader.clone()
    //}

    //pub fn get_load_request(&self) -> Sender<LoadRequest> {
    //    self.load_request.clone()
    //}

    pub fn terminate(mut self) {
        self.term();
        drop(self);
    }

    fn term(&mut self) {
        //self.render.terminate();
    }
}

impl Drop for MVCore {
    fn drop(&mut self) {
        self.term();
    }
}

impl Default for MVCore {
    fn default() -> Self {
        Self::new(ApplicationInfo::default())
    }
}

pub struct ApplicationInfo {
    name: String,
    version: Version
}

impl ApplicationInfo {
    fn new(name: &str, version: Version) -> ApplicationInfo {
        ApplicationInfo {
            name: name.to_string(),
            version
        }
    }
}

impl Default for ApplicationInfo {
    fn default() -> Self {
        ApplicationInfo {
            name: String::new(),
            version: Version::default()
        }
    }
}