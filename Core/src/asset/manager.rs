use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::task::Wake;
use std::thread::JoinHandle;
use ahash::AHasher;

use crossbeam_channel::{Receiver, Sender, unbounded};
use hashbrown::HashMap;
use mvsync::block::Signal;
use mvutils::hashers::U64IdentityHasher;
use parking_lot::{Mutex, RwLock};

use crate::asset::asset::{Asset, AssetType, InnerAsset};
use crate::asset::importer::AssetLoader;
use crate::render::backend::device::Device;

#[derive(Clone)]
pub struct AssetHandle {
    manager: Arc<AssetManager>,
    path: Arc<String>,
    handle: u64,
    global: bool,
    counter: Arc<Mutex<AtomicU64>>,
    progress: Arc<AtomicBool>,
    signal: Arc<Signal>,
}

impl AssetHandle {
    pub fn load<F: Fn(AssetHandle) + 'static>(&self, callback: F) {
        if self.global { return; }
        if self.counter.lock().fetch_add(1, Ordering::AcqRel) == 0 {
            self.progress.store(true, Ordering::Release);
            self.manager.push(AssetTask::Load(self.clone(), Box::new(callback)));
        }
    }

    pub fn unload<F: Fn(AssetHandle) + 'static>(&self, callback: F) {
        if self.global { return; }
        if self.counter.lock().fetch_sub(1, Ordering::AcqRel) == 1 {
            self.progress.store(true, Ordering::Release);
            self.manager.push(AssetTask::Unload(self.clone(), Box::new(callback)));
        }
    }

    pub fn reload<F: Fn(AssetHandle) + 'static>(&self, callback: F) {
        if self.counter.lock().load(Ordering::Acquire) > 0 {
            self.progress.store(true, Ordering::Release);
            self.manager.push(AssetTask::Load(self.clone(), Box::new(callback)));
        }
    }

    pub fn is_valid(&self) -> bool {
        self.manager.is_asset_handle_valid(self)
    }

    pub fn is_loaded(&self) -> bool {
        self.global || self.manager.is_asset_loaded(self)
    }

    pub fn wait(&self) {
        while !self.signal.ready() && self.progress.load(Ordering::Acquire) {
            self.signal.wait();
        }
    }

    pub async fn wait_async(&self) {
        self.signal.wait_async().await
    }

    pub fn get(&self) -> Arc<Asset> {
        self.manager.get(self)
    }

    pub fn get_manager(&self) -> Arc<AssetManager> {
        self.manager.clone()
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }
}

impl PartialEq for AssetHandle {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for AssetHandle {}

impl Hash for AssetHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.handle);
    }
}

pub struct AssetManager {
    asset_map: RwLock<HashMap<AssetHandle, Arc<Asset>, U64IdentityHasher>>,
    threads: Vec<(JoinHandle<()>, Sender<AssetTask>)>,
    index: AtomicU64,
    queued: Arc<AtomicU64>,
    loader: AssetLoader,
}

impl AssetManager {
    pub fn new(device: Device, thread_count: u64) -> Arc<Self> {
        assert!(thread_count > 0, "Asset manager thread count cannot be 0!");
        let mut threads = Vec::with_capacity(thread_count as usize);
        let queued = Arc::new(AtomicU64::new(0));
        for _ in 0..thread_count {
            let (sender, receiver) = unbounded();
            let queued = queued.clone();
            let thread = std::thread::spawn(|| Self::loader_thread(receiver, queued));
            threads.push((thread, sender));
        }
        
        Self {
            asset_map: RwLock::new(HashMap::with_hasher(U64IdentityHasher::default())),
            threads,
            index: AtomicU64::new(0),
            queued,
            loader: AssetLoader::new(device),
        }.into()
    }

    #[allow(invalid_reference_casting)]
    fn loader_thread(receiver: Receiver<AssetTask>, queued: Arc<AtomicU64>) {
        while let Ok(task) = receiver.recv() {
            match task {
                AssetTask::Load(handle, callback) => {
                    let asset = handle.get();
                    unsafe { &mut *(&*asset as *const Asset as *mut Asset) }.load();
                    queued.fetch_sub(1, Ordering::AcqRel);
                    callback(handle.clone());
                    handle.progress.store(false, Ordering::Release);
                    handle.signal.clone().wake();
                }
                AssetTask::Unload(handle, callback) => {
                    let asset = handle.get();
                    unsafe { &mut *(&*asset as *const Asset as *mut Asset) }.unload();
                    queued.fetch_sub(1, Ordering::AcqRel);
                    callback(handle.clone());
                    handle.progress.store(false, Ordering::Release);
                    handle.signal.clone().wake();
                }
                AssetTask::Reload(handle, callback) => {
                    let asset = handle.get();
                    unsafe { &mut *(&*asset as *const Asset as *mut Asset) }.reload();
                    queued.fetch_sub(1, Ordering::AcqRel);
                    callback(handle.clone());
                    handle.progress.store(false, Ordering::Release);
                    handle.signal.clone().wake();
                }
                AssetTask::Close => {
                    queued.fetch_sub(1, Ordering::AcqRel);
                    break
                },
            }
        }
    }

    pub fn create_asset(self: &Arc<Self>, path: &str, ty: AssetType) -> AssetHandle {
        self.create_asset_inner(path, ty, false)
    }

    pub fn create_global_asset(self: &Arc<Self>, path: &str, ty: AssetType) -> AssetHandle {
        self.create_asset_inner(path, ty, true)
    }

    fn create_asset_inner(self: &Arc<Self>, path: &str, ty: AssetType, global: bool) -> AssetHandle {
        let mut hasher = AHasher::default();
        path.hash(&mut hasher);
        let handle = hasher.finish();
        let handle = AssetHandle {
            manager: self.clone(),
            path: Arc::new(path.to_string()),
            handle,
            global,
            counter: Arc::new(AtomicU64::new(0).into()),
            progress: Arc::new(AtomicBool::new(false)),
            signal: Arc::new(Signal::new()),
        };
        let asset = Asset {
            inner: InnerAsset::Unloaded,
            ty,
            handle: handle.clone(),
        };
        self.asset_map.write().insert(handle.clone(), asset.into());
        if global {
            self.push(AssetTask::Load(handle.clone(), Box::new(|_| {})));
        }
        handle
    }

    pub fn get(&self, handle: &AssetHandle) -> Arc<Asset> {
        self.asset_map.read().get(handle).expect("AssetHandle not valid").clone()
    }

    pub fn is_asset_handle_valid(&self, handle: &AssetHandle) -> bool {
        self.asset_map.read().contains_key(handle)
    }

    pub fn is_asset_loaded(&self, handle: &AssetHandle) -> bool {
        self.asset_map.read().get(handle).map(|asset| asset.is_loaded()).unwrap_or_default()
    }

    fn push(&self, task: AssetTask) {
        let index = self.index.load(Ordering::Acquire);
        self.index.store((index + 1) % self.threads.len() as u64, Ordering::Release);
        self.queued.fetch_add(1, Ordering::AcqRel);
        #[allow(invalid_reference_casting)]
        if let Err(task) = self.threads[index as usize].1.send(task) {
            // Unsafe but not error-prone code, if the thread is down, we can safely replace it like
            // this, since we are not pushing to the Vec, so it cannot reallocate
            log::error!("Asset loading thread has dropped receiver, this means that it was probably killed or panicked. Starting new asset loader thread!");
            let (sender, receiver) = unbounded();
            sender.send(task.0).expect("Failed to send upon creation");
            let queued = self.queued.clone();
            let thread = std::thread::spawn(|| Self::loader_thread(receiver, queued));
            let _unsafe_mut = unsafe { &mut *(&self.threads as *const _ as *mut Vec<(JoinHandle<()>, Sender<AssetTask>)>) };
            _unsafe_mut[index as usize] = (thread, sender);
        }
    }

    pub fn get_queued(&self) -> u64 {
        self.queued.load(Ordering::Acquire)
    }

    pub fn get_loader(&self) -> AssetLoader {
        self.loader.clone()
    }
}

impl Drop for AssetManager {
    fn drop(&mut self) {
        for (handle, _) in self.asset_map.read().iter() {
            if handle.global || handle.counter.lock().load(Ordering::Acquire) > 0 {
                self.push(AssetTask::Unload(handle.clone(), Box::new(|_| {})));
            }
        }
        for (_, sender) in &self.threads {
            let _ = sender.send(AssetTask::Close);
        }

        for (thread, _) in self.threads.drain(..) {
            let _ = thread.join();
        }
    }
}

enum AssetTask {
    Load(AssetHandle, Box<dyn Fn(AssetHandle)>),
    Unload(AssetHandle, Box<dyn Fn(AssetHandle)>),
    Reload(AssetHandle, Box<dyn Fn(AssetHandle)>),
    Close,
}

unsafe impl Send for AssetTask {}