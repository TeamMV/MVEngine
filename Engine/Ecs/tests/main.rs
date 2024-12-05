use mvcore::math::vec::Vec3;
use mvengine_ecs::{Entity, ECS};
use mvengine_ecs::system::System;

#[derive(Default, Clone)]
struct Health {
    pub health: f32
}

#[derive(Default, Clone)]
struct Transform {
    pub pos: Vec3
}

fn main() {
    let ecs = ECS::new();

    let mut player = Entity::<(Health, Transform)>::new(ecs.storage());
    let health1 = player.get_component_mut::<Health>().unwrap();
    health1.health = 10f32;

    let mut player2 = player.clone();
    let health2 = player2.get_component_mut::<Health>().unwrap();
    health2.health = 5f32;

    let mut block = Entity::<(Transform,)>::new(ecs.storage());

    println!("Health2: {}", health2.health);

    let my_system = System::<(Transform, Health)>::new(ecs.storage());
    for (trans, health) in my_system.iter() {
        println!("Health component: {} - Transform component: {:?}", health.health, trans.pos);
    }
}