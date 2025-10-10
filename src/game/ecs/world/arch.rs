use crate::game::ecs::entity::EntityId;
use crate::game::ecs::world::EcsWorld;

pub struct ArchetypeWorld {}

impl EcsWorld for ArchetypeWorld {
    fn create_entity(&mut self, id: EntityId) {
        todo!()
    }

    fn destroy_entity(&mut self, id: EntityId) {
        todo!()
    }

    fn set_component<C: 'static>(&mut self, id: EntityId, c: C) {
        todo!()
    }

    fn get_component<C: 'static>(&self, id: EntityId) -> Option<&C> {
        todo!()
    }

    fn get_component_mut<C: 'static>(&mut self, id: EntityId) -> Option<&mut C> {
        todo!()
    }
}
