use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use include_dir::File;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use crate::assets::{AutomaticAssetManager, ManualAssetManager, ReadableAssetManager, SemiAutomaticAssetManager, WritableAssetManager};
use crate::old_render::load_render_assets;

pub enum AssetManager {
    Manual(ManualAssetManager),
    SemiAutomatic(SemiAutomaticAssetManager),
    Automatic(AutomaticAssetManager)
}

impl AssetManager {

    pub fn manual(manager: ManualAssetManager) -> AssetManager {
        AssetManager::Manual(manager)
    }

    pub fn semi_automatic(manager: SemiAutomaticAssetManager) -> AssetManager {
        AssetManager::SemiAutomatic(manager)
    }

    pub fn automatic(manager: AutomaticAssetManager) -> AssetManager {
        AssetManager::Automatic(manager)
    }

    fn is_writable(&self) -> bool {
        match self {
            AssetManager::Manual(_) => true,
            AssetManager::SemiAutomatic(_) => true,
            AssetManager::Automatic(_) => false
        }
    }

    fn is_automatic(&self) -> bool {
        match self {
            AssetManager::Manual(_) => false,
            AssetManager::SemiAutomatic(_) => true,
            AssetManager::Automatic(_) => true
        }
    }

    fn as_readable(&self) -> &dyn ReadableAssetManager {
        match self {
            AssetManager::Manual(manager) => manager,
            AssetManager::SemiAutomatic(manager) => manager,
            AssetManager::Automatic(manager) => manager
        }
    }

    fn as_writable(&self) -> &mut dyn WritableAssetManager {
        unsafe {
            let m = (self as *const AssetManager).cast_mut().as_mut().unwrap();
            match m {
                AssetManager::Manual(manager) => manager,
                AssetManager::SemiAutomatic(manager) => manager,
                AssetManager::Automatic(manager) => unreachable!()
            }
        }
    }

    fn raw(&self) -> &mut crate::assets::AssetManager {
        unsafe {
            let m = (self as *const AssetManager).cast_mut().as_mut().unwrap();
            match m {
                AssetManager::Manual(manager) => &mut manager.manager,
                AssetManager::SemiAutomatic(manager) => &mut manager.manager,
                AssetManager::Automatic(manager) => &mut manager.manager
            }
        }
    }
}

pub enum FontType {
    Bitmap(String, String),
    TrueType(String)
}

impl FontType {
    pub fn bitmap(bitmap: &str, fnt: &str) -> FontType {
        FontType::Bitmap(bitmap.to_string(), fnt.to_string())
    }

    pub fn true_type(ttf: &str) -> FontType {
        FontType::TrueType(ttf.to_string())
    }
}

pub enum ResourceLoadInfo {
    Shader(String, String),
    EffectShader(String),
    Texture(String),
    TextureRegion(String, u32, u32, u32, u32),
    FullTextureRegion(String),
    Font(FontType),
    Model(String)
}

impl ResourceLoadInfo {
    pub fn shader(vertex: &str, fragment: &str) -> ResourceLoadInfo {
        ResourceLoadInfo::Shader(vertex.to_string(), fragment.to_string())
    }

    pub fn effect_shader(fragment: &str) -> ResourceLoadInfo {
        ResourceLoadInfo::EffectShader(fragment.to_string())
    }

    pub fn texture(texture: &str) -> ResourceLoadInfo {
        ResourceLoadInfo::Texture(texture.to_string())
    }

    pub fn texture_region(texture: &str, x: u32, y: u32, width: u32, height: u32) -> ResourceLoadInfo {
        ResourceLoadInfo::TextureRegion(texture.to_string(), x, y, width, height)
    }

    pub fn full_texture_region(texture: &str) -> ResourceLoadInfo {
        ResourceLoadInfo::FullTextureRegion(texture.to_string())
    }

    pub fn font(font: FontType) -> ResourceLoadInfo {
        ResourceLoadInfo::Font(font)
    }

    pub fn model(model: &str) -> ResourceLoadInfo {
        ResourceLoadInfo::Model(model.to_string())
    }
}

pub enum ResponseMethod {
    Callback(Box<dyn Fn(Box<dyn LoadableResource>)>),
    Channel(Sender<Box<dyn LoadableResource>>)
}

impl ResponseMethod {
    pub fn callback(callback: Box<dyn Fn(Box<dyn LoadableResource>)>) -> ResponseMethod {
        ResponseMethod::Callback(callback)
    }

    pub fn channel(channel: Sender<Box<dyn LoadableResource>>) -> ResponseMethod {
        ResponseMethod::Channel(channel)
    }

    pub fn respond(self, resource: Box<dyn LoadableResource>) {
        match self {
            ResponseMethod::Callback(callback) => callback(resource),
            ResponseMethod::Channel(channel) => channel.send(resource).unwrap()
        }
    }
}

pub enum LoadRequest {
    ManagerResource(String, String, ResourceLoadInfo),
    CustomFileResource(String, String, Box<dyn Fn(File<'static>) -> Box<dyn LoadableResource>>, ResponseMethod),
    CustomMultiFileResource(String, Vec<String>, Box<dyn Fn(Vec<File<'static>>) -> Box<dyn LoadableResource>>, ResponseMethod),
    ExternalFile(Box<dyn Fn() -> Box<dyn LoadableResource>>, ResponseMethod),
    Function(Box<dyn Fn()>),
    AddManager(String, AssetManager)
}

unsafe impl Send for LoadRequest {}
unsafe impl Sync for LoadRequest {}

impl LoadRequest {
    pub fn manager_resource(manager_id: &str, resource_id: &str, info: ResourceLoadInfo) -> LoadRequest {
        LoadRequest::ManagerResource(manager_id.to_string(), resource_id.to_string(), info)
    }

    pub fn custom_file_resource(manager_id: &str, file_path: &str, loader: Box<dyn Fn(File<'static>) -> Box<dyn LoadableResource>>, response_method: ResponseMethod) -> LoadRequest {
        LoadRequest::CustomFileResource(manager_id.to_string(), file_path.to_string(), loader, response_method)
    }


    pub fn custom_multi_file_resource(manager_id: &str, file_paths: Vec<String>, loader: Box<dyn Fn(Vec<File<'static>>) -> Box<dyn LoadableResource>>, response_method: ResponseMethod) -> LoadRequest {
        LoadRequest::CustomMultiFileResource(manager_id.to_string(), file_paths, loader, response_method)
    }

    pub fn external_file(loader: Box<dyn Fn() -> Box<dyn LoadableResource>>, response_method: ResponseMethod) -> LoadRequest {
        LoadRequest::ExternalFile(loader, response_method)
    }

    pub fn function(function: Box<dyn Fn()>) -> LoadRequest {
        LoadRequest::Function(function)
    }

    pub fn add_manager(manager_id: &str, manager: AssetManager) -> LoadRequest {
        LoadRequest::AddManager(manager_id.to_string(), manager)
    }

    fn is_external(&self) -> bool {
        match self {
            LoadRequest::ExternalFile(..) => true,
            LoadRequest::Function(..) => true,
            _ => false
        }
    }

    fn is_modifying(&self) -> bool {
        match self {
            LoadRequest::ManagerResource(..) => true,
            LoadRequest::AddManager(..) => true,
            _ => false
        }
    }
}

pub trait LoadableResource {}

impl<T> LoadableResource for T {}

pub struct ResourceLoader {
    asset_managers: HashMap<String, AssetManager>,
    load_requests: Receiver<LoadRequest>,
    load_sender: Sender<LoadRequest>
}

unsafe impl Send for ResourceLoader {}
unsafe impl Sync for ResourceLoader {}

impl ResourceLoader {
    pub(crate) fn new() -> (ResourceLoader) {
        let (load_sender, load_requests) = channel();
        ResourceLoader {
            asset_managers: HashMap::new(),
            load_requests,
            load_sender
        }
    }

    pub fn get_asset_manager(&self, name: &str) -> &dyn ReadableAssetManager {
        self.asset_managers.get(name).map(|m| m.as_readable()).unwrap()
    }

    pub fn try_get_asset_manager(&self, name: &str) -> Option<&dyn ReadableAssetManager> {
        self.asset_managers.get(name).map(|m| m.as_readable())
    }

    pub(crate) fn load(&self) {
        load_render_assets(self.asset_managers.get("MVCore").unwrap().as_writable());
        for manager in self.asset_managers.values() {
            match manager {
                AssetManager::SemiAutomatic(manager) => {

                }
                AssetManager::Automatic(manager) => {

                }
                _ => {}
            }
        }
    }

    pub fn get_load_request(&self) -> Sender<LoadRequest> {
        self.load_sender.clone()
    }

    pub(crate) fn start(&mut self) {
        unsafe {
            fn require(workers: &mut [Option<JoinHandle<()>>; 5]) -> usize {
                loop {
                    for i in 0..5 {
                        if let Some(w) = &workers[i] {
                            if w.is_finished() {
                                workers[i] = None;
                                return i;
                            }
                        } else {
                            return i;
                        }
                    }
                }
            }

            fn require_freed(workers: &mut [Option<JoinHandle<()>>; 5], runtime: &Runtime) {
                runtime.block_on(async move {
                    tokio::join!(
                        workers[0].take().unwrap_or_else(|| runtime.spawn(async{})),
                        workers[1].take().unwrap_or_else(|| runtime.spawn(async{})),
                        workers[2].take().unwrap_or_else(|| runtime.spawn(async{})),
                        workers[3].take().unwrap_or_else(|| runtime.spawn(async{})),
                        workers[4].take().unwrap_or_else(|| runtime.spawn(async{}))
                    );
                });
            }

            let runtime = Runtime::new().expect("Could not create resource loader runtime!");
            let mut self_workers: [Option<JoinHandle<()>>; 5] = [0; 5].map(|_| None);
            let mut workers: [Option<JoinHandle<()>>; 5] = [0; 5].map(|_| None);
            let ptr = self as *const ResourceLoader;
            while let Ok(req) = self.load_requests.recv() {
                if req.is_external() {
                    let id = require(&mut workers);
                    workers[id] = Some(runtime.spawn(Self::process_external(req)));
                } else if req.is_modifying() {
                    require_freed(&mut self_workers, &runtime);
                    (ptr.cast_mut().as_mut().unwrap()).process_modifying(req);
                } else {
                    let id = require(&mut self_workers);
                    let task = (ptr.as_ref().unwrap()).process(req);
                    self_workers[id] = Some(runtime.spawn(task));
                }
            }
        }
    }

    fn process_modifying(&mut self, req: LoadRequest) {
        match req {
            LoadRequest::ManagerResource(manager, id, info) => {
                let m = self.asset_managers.get(manager.as_str()).expect(format!("Manager '{}' not found!", manager).as_str());
                if !m.is_writable() {
                    panic!("Tried to manually load for an automatic manager '{}'. Consider using a semi-automatic manager instead.", manager);
                }
                let manager = m.as_writable();
                match info {
                    ResourceLoadInfo::Shader(vert, frag) => {
                        manager.load_shader(id.as_str(), vert.as_str(), frag.as_str());
                    }
                    ResourceLoadInfo::EffectShader(frag) => {
                        manager.load_effect_shader(id.as_str(), frag.as_str());
                    }
                    ResourceLoadInfo::Texture(tex) => {
                        manager.load_texture(id.as_str(), tex.as_str());
                    }
                    ResourceLoadInfo::TextureRegion(tex, x, y, w, h) => {
                        manager.crop_texture_region(id.as_str(), tex.as_str(), x, y, w, h);
                    }
                    ResourceLoadInfo::FullTextureRegion(tex) => {
                        manager.prepare_texture(id.as_str(), tex.as_str());
                    }
                    ResourceLoadInfo::Font(font) => {
                        match font {
                            FontType::Bitmap(bitmap, fnt) => {
                                manager.load_bitmap_font(id.as_str(), bitmap.as_str(), fnt.as_str());
                            }
                            FontType::TrueType(ttf) => {
                                manager.load_ttf_font(id.as_str(), ttf.as_str());
                            }
                        }
                    }
                    ResourceLoadInfo::Model(model) => {
                        manager.load_model(id.as_str(), model.as_str());
                    }
                }
            }
            LoadRequest::AddManager(name, manager) => {
                if !self.asset_managers.contains_key(name.as_str()) {
                    self.asset_managers.insert(name, manager);
                }
            }
            _ => {}
        }
    }

    async fn process(&self, req: LoadRequest) {
        match req {
            LoadRequest::CustomFileResource(manager, path, loader, response) => {
                let manager = self.asset_managers.get(manager.as_str()).expect(format!("Manager '{}' not found!", manager).as_str());
                let file = manager.raw().files.remove(path.as_str()).expect(format!("File '{}' not found!", path).as_str());
                response.respond(loader(file));
            }
            LoadRequest::CustomMultiFileResource(manager, paths, loader, response) => {
                let manager = self.asset_managers.get(manager.as_str()).expect(format!("Manager '{}' not found!", manager).as_str());
                let files = paths.into_iter().map(|s| manager.raw().files.remove(s.as_str()).expect(format!("File '{}' not found!", s).as_str())).collect();
                response.respond(loader(files));
            }
            _ => {}
        }
    }

    async fn process_external(req: LoadRequest) {
        match req {
            LoadRequest::ExternalFile(loader, response) => {
                response.respond(Box::new(loader()));
            }
            LoadRequest::Function(loader) => {
                loader();
            }
            _ => {}
        }
    }
}