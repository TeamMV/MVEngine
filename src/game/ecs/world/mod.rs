pub mod sparse;
pub mod arch;

use crate::game::ecs::entity::EntityId;
pub trait EcsWorld {
    fn create_entity(&mut self, id: EntityId);
    fn destroy_entity(&mut self, id: EntityId);

    fn set_component<C: 'static>(&mut self, id: EntityId, c: C);
    fn get_component<C: 'static>(&self, id: EntityId) -> Option<&C>;
    fn get_component_mut<C: 'static>(&mut self, id: EntityId) -> Option<&mut C>;
}