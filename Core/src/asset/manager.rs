use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::JoinHandle;

use crossbeam_channel::{Receiver, Sender, unbounded};
use hashbrown::HashMap;
use mvutils::hashers::U64IdentityHasher;
use parking_lot::{Mutex, RwLock};

use crate::asset::asset::{Asset, AssetData, AssetType};

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
            self.manager.push(AssetTask::Load(self.clone()), 0);
        }
    }

    pub fn unload(&self) {
        if self.global { return; }
        if self.counter.lock().fetch_sub(1, Ordering::AcqRel) == 1 {
            self.manager.push(AssetTask::Unload(self.clone()), 0);
        }
    }

    pub fn is_valid(&self) -> bool {
        self.manager.is_asset_handle_valid(self)
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
}

impl AssetManager {
    pub fn new(thread_count: u64) -> Arc<Self> {
        let mut threads = Vec::with_capacity(thread_count as usize);
        for _ in 0..thread_count {
            let (sender, receiver) = unbounded();
            let thread = std::thread::spawn(|| Self::loader_thread(receiver));
            threads.push((thread, sender));
        }
        
        Self {
            asset_map: RwLock::new(HashMap::with_hasher(U64IdentityHasher::default())),
            threads,
            index: AtomicU64::new(0),
            handle: AtomicU64::new(0),
        }.into()
    }

    #[allow(invalid_reference_casting)]
    fn loader_thread(receiver: Receiver<AssetTask>) {
        while let Ok(task) = receiver.recv() {
            match task {
                AssetTask::Load(handle) => {
                    let asset = handle.get();
                    unsafe { &mut *(&*asset as *const Asset as *mut Asset) }.load();
                }
                AssetTask::Unload(handle) => {
                    let asset = handle.get();
                    unsafe { &mut *(&*asset as *const Asset as *mut Asset) }.unload();
                }
                AssetTask::Close => break,
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
        let asset = Asset::Unloaded(AssetData {
            ty,
            path,
        });
        let handle = self.handle.fetch_add(1, Ordering::AcqRel);
        let handle = AssetHandle {
            manager: self.clone(),
            handle,
            global,
            counter: Arc::new(AtomicU64::new(0).into()),
        };
        self.asset_map.write().insert(handle.clone(), asset.into());
        if global {
            self.push(AssetTask::Load(handle.clone()), 0);
        }
        handle
    }

    pub fn get(&self, handle: &AssetHandle) -> Arc<Asset> {
        self.asset_map.read().get(handle).expect("AssetHandle not valid").clone()
    }

    pub fn is_asset_handle_valid(&self, handle: &AssetHandle) -> bool {
        self.asset_map.read().contains_key(handle)
    }

    fn push(&self, task: AssetTask, depth: u64) {
        let index = self.index.load(Ordering::Acquire);
        self.index.store((index + 1) % self.threads.len() as u64, Ordering::Release);
        // basically try all threads in case one crashes or something
        // TODO: refactor this to perhaps try reboot the broken thread
        if let Err(task) = self.threads[index as usize].1.send(task) {
            if depth < self.threads.len() as u64 {
                self.push(task.0, depth + 1);
            }
        }
    }
}

impl Drop for AssetManager {
    fn drop(&mut self) {
        for (handle, _) in self.asset_map.read().iter() {
            if handle.global || handle.counter.lock().load(Ordering::Acquire) > 0 {
                self.push(AssetTask::Unload(handle.clone()), 0);
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
    Close,
}