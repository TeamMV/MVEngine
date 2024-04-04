use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::JoinHandle;
use crossbeam_channel::Sender;

use hashbrown::HashMap;
use mvutils::hashers::U64IdentityHasher;
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use parking_lot::{Mutex, RwLock};

use crate::asset::asset::Asset;

#[derive(Clone, Debug)]
pub struct AssetHandle {
    manager: Arc<AssetManager>,
    handle: u64,
    counter: Arc<Mutex<AtomicU64>>,
}

impl AssetHandle {
    pub fn load(&self) {
        if self.counter.lock().fetch_add(1, Ordering::AcqRel) == 0 {
            self.manager.push(AssetTask::Load(self.clone()), 0);
        }
    }

    pub fn unload(&self) {
        if self.counter.lock().fetch_sub(1, Ordering::AcqRel) == 1 {
            self.manager.push(AssetTask::Unload(self.clone()), 0);
        }
    }

    pub fn is_valid(&self) -> bool {
        self.manager.is_asset_handle_valid(self)
    }

    pub fn get_asset(&self) -> &Asset {
        self.manager.get_asset(self)
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
    asset_map: RwLock<HashMap<AssetHandle, DangerousCell<Asset>, U64IdentityHasher>>,
    threads: Vec<(JoinHandle<()>, Sender<AssetTask>)>,
    index: AtomicU64,
}

impl AssetManager {
    pub fn new() -> Arc<Self> {
        todo!()
    }

    pub fn create_asset(self: &Arc<Self>, path: String) -> AssetHandle {
        todo!()
    }

    pub fn get_asset(&self, handle: &AssetHandle) -> &Asset {
        let guard = self.asset_map.read();
        let asset = unsafe { Unsafe::cast_static(guard.get(handle).expect("AssetHandle not valid").get()) };
        drop(guard);
        asset
    }

    pub fn is_asset_handle_valid(&self, handle: &AssetHandle) -> bool {
        todo!()
    }

    fn push(&self, task: AssetTask, depth: u64) {
        let index = self.index.load(Ordering::AcqRel);
        self.index.store((index + 1) % self.threads.len() as u64, Ordering::AcqRel);
        // basically try all threads in case one crashes or something
        // TODO: refactor this to perhaps try reboot the broken thread
        if let Err(task) = self.threads[index as usize].1.send(task) {
            if depth < self.threads.len() as u64 {
                self.push(task.0, depth + 1);
            }
        }
    }
}

enum AssetTask {
    Load(AssetHandle),
    Unload(AssetHandle),
    Close,
}