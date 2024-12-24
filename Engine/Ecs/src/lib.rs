use std::marker::PhantomData;
use std::sync::Arc;
use mvutils::unsafe_utils::DangerousCell;
use mvutils::utils;
use crate::mem::storage::ComponentStorage;

mod mem;
pub mod system;
pub mod entity;

pub type EcsStorage = Arc<DangerousCell<ComponentStorage>>;

pub struct ECS {
    pub(crate) storage: EcsStorage,
}

impl ECS {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(DangerousCell::new(ComponentStorage::new()))
        }
    }

    pub fn storage(&self) -> EcsStorage {
        self.storage.clone()
    }
}