use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::task::Wake;
use std::thread::JoinHandle;
use ahash::AHasher;
use crossbeam_channel::{unbounded, Receiver, Sender};
use hashbrown::HashMap;
use mvsync::block::Signal;
use mvutils::hashers::U64IdentityHasher;
use parking_lot::{Mutex, RwLock};

use crate::asset::asset::{Asset, AssetType, InnerAsset};
use crate::asset::importer::AssetLoader;
use crate::render::backend::device::Device;

pub type AssetCallback = Arc<dyn Fn(AssetHandle, u32)>;

struct AssetHandleInner {
    path: String,
    counter: Mutex<AtomicU64>,
    progress: AtomicBool,
}

#[derive(Clone)]
pub struct AssetHandle {
    manager: Arc<AssetManager>,
    signal: Arc<Signal>,
    inner: Arc<AssetHandleInner>,
    handle: u64,
    global: bool,
}

impl AssetHandle {
    pub fn load<F: Fn(AssetHandle, u32) + 'static>(&self, callback: F) {
        if self.global { return; }
        if self.inner.counter.lock().fetch_add(1, Ordering::AcqRel) == 0 {
            self.inner.progress.store(true, Ordering::Release);
            self.manager.push(AssetTask::Load(self.clone(), Arc::new(callback)));
        }
    }

    pub fn unload<F: Fn(AssetHandle, u32) + 'static>(&self, callback: F) {
        if self.global { return; }
        if self.inner.counter.lock().fetch_sub(1, Ordering::AcqRel) == 1 {
            self.inner.progress.store(true, Ordering::Release);
            self.manager.push(AssetTask::Unload(self.clone(), Arc::new(callback)));
        }
    }

    pub fn reload<F: Fn(AssetHandle, u32) + 'static>(&self, callback: F) {
        if self.inner.counter.lock().load(Ordering::Acquire) > 0 {
            self.inner.progress.store(true, Ordering::Release);
            self.manager.push(AssetTask::Load(self.clone(), Arc::new(callback)));
        }
    }

    pub fn is_valid(&self) -> bool {
        self.manager.is_asset_handle_valid(self)
    }

    pub fn is_loaded(&self) -> bool {
        self.global || self.manager.is_asset_loaded(self)
    }

    pub fn wait(&self) {
        while !self.signal.ready() && self.inner.progress.load(Ordering::Acquire) {
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
        &self.inner.path
    }
}

impl PartialEq for AssetHandle {
    fn eq(&self, other: &Self) -> bool {
        self.inner.path == other.inner.path
    }
}

impl Eq for AssetHandle {}

impl Hash for AssetHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.handle);
    }
}

unsafe impl Send for AssetHandle {}
unsafe impl Sync for AssetHandle {}

pub struct AssetManager {
    asset_map: RwLock<HashMap<AssetHandle, Arc<Asset>, U64IdentityHasher>>,
    threads: Vec<(JoinHandle<()>, Sender<AssetTask>)>,
    index: AtomicU64,
    queued: Arc<AtomicU64>,
    loader: AssetLoader,
    queue: Arc<Mutex<Vec<VecDeque<(AssetCallback, AssetHandle)>>>>,
}

impl AssetManager {
    pub fn new(device: Device, thread_count: u64, queue_count: u32) -> Arc<Self> {
        assert!(thread_count > 0, "Asset manager thread count cannot be 0!");
        let queued = Arc::new(AtomicU64::new(0));

        let mut queues = Vec::with_capacity(queue_count as usize);
        for i in 0..queue_count {
            queues.push(VecDeque::new())
        }

        let queue = Arc::new(Mutex::new(queues));

        let manager = Arc::new(Self {
            asset_map: RwLock::new(HashMap::with_hasher(U64IdentityHasher::default())),
            threads: Vec::with_capacity(thread_count as usize),
            index: AtomicU64::new(0),
            queued: queued.clone(),
            loader: AssetLoader::new(device),
            queue: queue.clone(),
        });

        #[allow(invalid_reference_casting)]
        let threads = unsafe { &mut *(&manager.threads as *const _ as *mut Vec<(JoinHandle<()>, Sender<AssetTask>)>) };

        for _ in 0..thread_count {
            let (sender, receiver) = unbounded();
            let queued = queued.clone();
            let queue = queue.clone();
            let manager = manager.clone();
            let thread = std::thread::spawn(move || Self::loader_thread(receiver, manager));
            threads.push((thread, sender));
        }

        manager
    }

    #[allow(invalid_reference_casting)]
    fn loader_thread(receiver: Receiver<AssetTask>, manager: Arc<AssetManager>) {
        while let Ok(task) = receiver.recv() {
            match task {
                AssetTask::Load(handle, callback) => {
                    let asset = handle.get();
                    unsafe { &mut *(&*asset as *const Asset as *mut Asset) }.load();
                    manager.queued.fetch_sub(1, Ordering::AcqRel);
                    handle.inner.progress.store(false, Ordering::Release);
                    handle.signal.clone().wake();
                    for queue in manager.queue.lock().iter_mut() {
                        queue.push_back((callback.clone(), handle.clone()))
                    }
                }
                AssetTask::Unload(handle, callback) => {
                    let asset = handle.get();
                    unsafe { &mut *(&*asset as *const Asset as *mut Asset) }.unload();
                    manager.queued.fetch_sub(1, Ordering::AcqRel);
                    handle.inner.progress.store(false, Ordering::Release);
                    handle.signal.clone().wake();
                    for queue in manager.queue.lock().iter_mut() {
                        queue.push_back((callback.clone(), handle.clone()))
                    }
                }
                AssetTask::Reload(handle, callback) => {
                    let asset = handle.get();
                    unsafe { &mut *(&*asset as *const Asset as *mut Asset) }.reload();
                    manager.queued.fetch_sub(1, Ordering::AcqRel);
                    handle.inner.progress.store(false, Ordering::Release);
                    handle.signal.clone().wake();
                    for queue in manager.queue.lock().iter_mut() {
                        queue.push_back((callback.clone(), handle.clone()))
                    }
                }
                AssetTask::Close => {
                    manager.queued.fetch_sub(1, Ordering::AcqRel);
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

    fn create_asset_inner(
        self: &Arc<Self>,
        path: &str,
        ty: AssetType,
        global: bool,
    ) -> AssetHandle {
        let mut hasher = AHasher::default();
        path.hash(&mut hasher);
        let handle = hasher.finish();
        let handle = AssetHandle {
            manager: self.clone(),
            signal: Arc::new(Signal::new()),
            inner: Arc::new(AssetHandleInner {
                path: path.to_string(),
                counter: AtomicU64::new(0).into(),
                progress: AtomicBool::new(false),
            }),
            handle,
            global,
        };
        let asset = Asset {
            inner: InnerAsset::Unloaded,
            ty,
            handle: handle.clone(),
        };
        self.asset_map.write().insert(handle.clone(), asset.into());
        if global {
            self.push(AssetTask::Load(handle.clone(), Arc::new(|_, _| {})));
        }
        handle
    }

    pub fn get(&self, handle: &AssetHandle) -> Arc<Asset> {
        self.asset_map
            .read()
            .get(handle)
            .expect("AssetHandle not valid")
            .clone()
    }

    pub fn is_asset_handle_valid(&self, handle: &AssetHandle) -> bool {
        self.asset_map.read().contains_key(handle)
    }

    pub fn is_asset_loaded(&self, handle: &AssetHandle) -> bool {
        self.asset_map
            .read()
            .get(handle)
            .map(|asset| asset.is_loaded())
            .unwrap_or_default()
    }

    fn push(self: &Arc<AssetManager>, task: AssetTask) {
        let index = self.index.load(Ordering::Acquire);
        self.index
            .store((index + 1) % self.threads.len() as u64, Ordering::Release);
        self.queued.fetch_add(1, Ordering::AcqRel);
        #[allow(invalid_reference_casting)]
        if let Err(task) = self.threads[index as usize].1.send(task) {
            // Unsafe but not error-prone code, if the thread is down, we can safely replace it like
            // this, since we are not pushing to the Vec, so it cannot reallocate
            log::error!("Asset loading thread has dropped receiver, this means that it was probably killed or panicked. Starting new asset loader thread!");
            let (sender, receiver) = unbounded();
            sender.send(task.0).expect("Failed to send upon creation");
            let queued = self.queued.clone();
            let queue = self.queue.clone();
            let manager = self.clone();
            let thread = std::thread::spawn(|| Self::loader_thread(receiver, manager));
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

    pub fn queue_count(&self) -> u32 {
        self.queue.lock().len() as u32
    }

    pub fn set_queue_count(&self, queue_count: u32) {
        let mut queues = self.queue.lock();
        if queue_count as usize == queues.len() { return; }
        queues.clear();
        for i in 0..queue_count {
            queues.push(VecDeque::new())
        }
    }

    pub fn poll_queue(&self, index: u32) {
        let mut queues = self.queue.lock();
        if index as usize >= queues.len() { return; }
        for (task, handle) in queues[index as usize].drain(..) {
            task(handle, index);
        }
    }
}

impl Drop for AssetManager {
    fn drop(&mut self) {
        for (handle, asset) in self.asset_map.read().iter() {
            if handle.global || handle.inner.counter.lock().load(Ordering::Acquire) > 0 {
                let index = self.index.load(Ordering::Acquire);
                self.index.store((index + 1) % self.threads.len() as u64, Ordering::Release);
                self.queued.fetch_add(1, Ordering::AcqRel);
                if let Err(_) = self.threads[index as usize].1.send(AssetTask::Unload(handle.clone(), Arc::new(|_, _| {}))) {
                    #[allow(invalid_reference_casting)]
                    unsafe { &mut *(&**asset as *const Asset as *mut Asset) }.unload();
                }
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

unsafe impl Send for AssetManager {}
unsafe impl Sync for AssetManager {}

enum AssetTask {
    Load(AssetHandle, AssetCallback),
    Unload(AssetHandle, AssetCallback),
    Reload(AssetHandle, AssetCallback),
    Close,
}

unsafe impl Send for AssetTask {}
unsafe impl Sync for AssetTask {}