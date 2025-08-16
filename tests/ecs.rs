use mvengine::game::ecs::{Ecs, EcsBackend};
use mvengine::game::ecs::entity::Entity;
use mvengine::game::ecs::system::System;
use mvengine::math::vec::Vec2;

#[derive(Clone, Default, Debug)]
struct Transform {
    pos: Vec2
}

#[derive(Clone, Default)]
struct Velocity {
    vel: Vec2
}

fn main() {
    let mut ecs = Ecs::new(EcsBackend::SparseSet);
    let world = ecs.world_mut();
    let en = Entity::<(Transform, Velocity)>::create(world);
    let en = Entity::<(Transform, Velocity)>::create(world);
    let en = Entity::<(Transform, Velocity)>::create(world);

    let sys = System::<(Transform,)>::new();
    for (entity, transform) in sys.iter(world) {
        println!("{entity}: {transform:?}")
    }
    println!("end");
}