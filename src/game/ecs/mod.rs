use crate::game::ecs::entity::EntityId;
use crate::game::ecs::mem::storage::ComponentStorage;
use crate::game::ecs::world::EcsWorld;
use crate::game::ecs::world::arch::ArchetypeWorld;
use world::sparse::SparseSetWorld;

pub mod entity;
pub mod mem;
pub mod system;
pub mod world;

pub enum EcsBackend {
    SparseSet,
    Archetype,
}

pub struct Ecs {
    world: World,
}

impl Ecs {
    pub fn new(backend: EcsBackend) -> Self {
        Self {
            world: World::new(backend),
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

pub enum World {
    SparseSet(SparseSetWorld),
    ArchetypeWorld(ArchetypeWorld),
}

impl World {
    pub(crate) fn new(backend: EcsBackend) -> Self {
        match backend {
            EcsBackend::SparseSet => World::SparseSet(SparseSetWorld::new(ComponentStorage::new())),
            EcsBackend::Archetype => World::ArchetypeWorld(ArchetypeWorld {}),
        }
    }
}

macro_rules! world_fn {
    ($this:ident, $fn_name:ident()) => {
        match $this {
            World::SparseSet(e) => e.$fn_name(),
            World::ArchetypeWorld(e) => e.$fn_name(),
        }
    };
    ($this:ident, $fn_name:ident($($args:ident),*)) => {
        match $this {
            World::SparseSet(e) => e.$fn_name($($args),*),
            World::ArchetypeWorld(e) => e.$fn_name($($args),*),
        }
    };
}

impl EcsWorld for World {
    fn create_entity(&mut self, id: EntityId) {
        world_fn!(self, create_entity(id))
    }

    fn destroy_entity(&mut self, id: EntityId) {
        world_fn!(self, destroy_entity(id))
    }

    fn set_component<C: 'static>(&mut self, id: EntityId, c: C) {
        world_fn!(self, set_component(id, c));
    }

    fn get_component<C: 'static>(&self, id: EntityId) -> Option<&C> {
        world_fn!(self, get_component(id))
    }

    fn get_component_mut<C: 'static>(&mut self, id: EntityId) -> Option<&mut C> {
        world_fn!(self, get_component_mut(id))
    }
}
