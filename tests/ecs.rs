use mvengine::game::ecs::entity::Entity;
use mvengine::game::ecs::world::EcsWorld;
use mvengine::game::ecs::{Ecs, EcsBackend};
use mvengine::game::physics::components::{AABBCollider, RigidDynamic, Transform};
use mvengine::game::physics::systems::PhysicsSystem;
use mvengine::math::vec::Vec2;
use std::sync::Arc;

fn main() {
    let mut ecs = Ecs::new(EcsBackend::SparseSet);
    let world = ecs.world_mut();
    let en1 = Entity::<(Transform, AABBCollider)>::create(world);
    let en2 = Entity::<(Transform, AABBCollider)>::create(world);
    let en3 = Entity::<(Transform, AABBCollider, RigidDynamic)>::create(world);

    if let Some(t1) = world.get_component_mut::<Transform>(en1) {
        t1.position = Vec2::new(5.0, 5.0);
    }

    let mut physics = PhysicsSystem::new();
    physics.iterate(world, 1.0);
    println!("end");
}
