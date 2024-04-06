use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::JoinHandle;

use crossbeam_channel::{Receiver, Sender, unbounded};
use hashbrown::HashMap;
use mvutils::hashers::U64IdentityHasher;
use parking_lot::{Mutex, RwLock};

use crate::asset::asset::{Asset, AssetData, AssetType, InnerAsset};

#[derive(Clone)]
pub struct AssetHandle {
    manager: Arc<AssetManager>,
    handle: u64,
    global: bool,
    counter: Arc<Mutex<AtomicU64>>,
}

impl AssetHandle {
    pub fn load(&self) {
        if self.global { return; }
        if self.counter.lock().fetch_add(1, Ordering::AcqRel) == 0 {
            self.manager.push(AssetTask::Load(self.clone()));
        }
    }

    pub fn unload(&self) {
        if self.global { return; }
        if self.counter.lock().fetch_sub(1, Ordering::AcqRel) == 1 {
            self.manager.push(AssetTask::Unload(self.clone()));
        }
    }

    pub fn reload(&self) {
        if self.counter.lock().load(Ordering::Acquire) > 0 {
            self.manager.push(AssetTask::Load(self.clone()));
        }
    }

    pub fn is_valid(&self) -> bool {
        self.manager.is_asset_handle_valid(self)
    }

    pub fn is_loaded(&self) -> bool {
        self.global || self.manager.is_asset_loaded(self)
    }

    pub fn get(&self) -> Arc<Asset> {
        self.manager.get(self)
    }
}

impl PartialEq for AssetHandle {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
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
    handle: AtomicU64,
    queued: Arc<AtomicU64>,
}

impl AssetManager {
    pub fn new(thread_count: u64) -> Arc<Self> {
        let mut threads = Vec::with_capacity(thread_count as usize);
        let queued = Arc::new(AtomicU64::new(0));
        for _ in 0..thread_count {
            let (sender, receiver) = unbounded();
            let thread = std::thread::spawn(|| Self::loader_thread(receiver, queued.clone()));
            threads.push((thread, sender));
        }
        
        Self {
            asset_map: RwLock::new(HashMap::with_hasher(U64IdentityHasher::default())),
            threads,
            index: AtomicU64::new(0),
            handle: AtomicU64::new(0),
            queued,
        }.into()
    }

    #[allow(invalid_reference_casting)]
    fn loader_thread(receiver: Receiver<AssetTask>, queued: Arc<AtomicU64>) {
        while let Ok(task) = receiver.recv() {
            match task {
                AssetTask::Load(handle) => {
                    let asset = handle.get();
                    unsafe { &mut *(&*asset as *const Asset as *mut Asset) }.load();
                    queued.fetch_sub(1, Ordering::AcqRel);
                }
                AssetTask::Unload(handle) => {
                    let asset = handle.get();
                    unsafe { &mut *(&*asset as *const Asset as *mut Asset) }.unload();
                    queued.fetch_sub(1, Ordering::AcqRel);
                }
                AssetTask::Reload(handle) => {
                    let asset = handle.get();
                    unsafe { &mut *(&*asset as *const Asset as *mut Asset) }.reload();
                    queued.fetch_sub(1, Ordering::AcqRel);
                }
                AssetTask::Close => {
                    queued.fetch_sub(1, Ordering::AcqRel);
                    break
                },
            }
        }
    }

    pub fn create_asset(self: &Arc<Self>, path: String, ty: AssetType) -> AssetHandle {
        self.create_asset_inner(path, ty, false)
    }

    pub fn create_global_asset(self: &Arc<Self>, path: String, ty: AssetType) -> AssetHandle {
        self.create_asset_inner(path, ty, true)
    }

    fn create_asset_inner(self: &Arc<Self>, path: String, ty: AssetType, global: bool) -> AssetHandle {
        let handle = self.handle.fetch_add(1, Ordering::AcqRel);
        let handle = AssetHandle {
            manager: self.clone(),
            handle,
            global,
            counter: Arc::new(AtomicU64::new(0).into()),
        };
        let asset = Asset {
            inner: InnerAsset::Unloaded,
            data: AssetData {
                ty,
                path
            },
            handle: handle.clone(),
        };
        self.asset_map.write().insert(handle.clone(), asset.into());
        if global {
            self.push(AssetTask::Load(handle.clone()));
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
            let thread = std::thread::spawn(|| Self::loader_thread(receiver, self.queued.clone()));
            let _unsafe_mut = unsafe { &mut *(&self.threads as *const _ as *mut Vec<(JoinHandle<()>, Sender<AssetTask>)>) };
            _unsafe_mut[index as usize] = (thread, sender);
        }
    }

    pub fn get_queued(&self) -> u64 {
        self.queued.load(Ordering::Acquire)
    }
}

impl Drop for AssetManager {
    fn drop(&mut self) {
        for (handle, _) in self.asset_map.read().iter() {
            if handle.global || handle.counter.lock().load(Ordering::Acquire) > 0 {
                self.push(AssetTask::Unload(handle.clone()));
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
    Load(AssetHandle),
    Unload(AssetHandle),
    Reload(AssetHandle),
    Close,
}