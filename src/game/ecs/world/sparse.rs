use crate::game::ecs::entity::EntityId;
use crate::game::ecs::mem::storage::ComponentStorage;
use crate::game::ecs::world::EcsWorld;

pub struct SparseSetWorld {
    pub(crate) storage: ComponentStorage,
    entities: Vec<EntityId>,
}

impl SparseSetWorld {
    pub(crate) fn new(storage: ComponentStorage) -> Self {
        Self {
            storage,
            entities: Vec::new(),
        }
    }
}

impl EcsWorld for SparseSetWorld {
    fn create_entity(&mut self, id: EntityId) {
        self.entities.push(id);
    }

    fn destroy_entity(&mut self, id: EntityId) {
        self.storage.remove_entity(id);
        if let Ok(idx) = self.entities.binary_search(&id) {
            self.entities.remove(idx);
        }
    }

    fn set_component<C: 'static>(&mut self, id: EntityId, c: C) {
        self.storage.set_component::<C>(id, c);
    }

    fn get_component<C: 'static>(&self, id: EntityId) -> Option<&C> {
        self.storage.get_component::<C>(id)
    }

    fn get_component_mut<C: 'static>(&mut self, id: EntityId) -> Option<&mut C> {
        self.storage.get_component_mut::<C>(id)
    }
}