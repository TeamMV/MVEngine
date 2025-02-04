#![feature(ptr_metadata)]

use std::marker::PhantomData;
use std::sync::Arc;
use mvutils::unsafe_utils::DangerousCell;
use mvutils::utils;
use crate::ecs::mem::storage::ComponentStorage;
use crate::ecs::world::World;

mod mem;
pub mod system;
pub mod entity;
pub mod world;

pub type EcsStorage = Arc<DangerousCell<ComponentStorage>>;

pub struct ECS {
    pub(crate) storage: EcsStorage,
    world: World
}

impl ECS {
    pub fn new() -> Self {
        let st = Arc::new(DangerousCell::new(ComponentStorage::new()));
        Self {
            storage: st.clone(),
            world: World::new(st),
        }
    }

    pub fn storage(&self) -> EcsStorage {
        self.storage.clone()
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}