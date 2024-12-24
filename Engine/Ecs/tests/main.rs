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

    let mut block1 = Entity::<(Transform,)>::new(ecs.storage());
    let trns1 = block1.get_component_mut::<Transform>().unwrap();
    trns1.pos = Vec3::splat(1.0);


    let my_system = System::<(Transform, Health)>::new(ecs.storage());
    for (en, transform, health) in my_system.iter() {
        println!("entity: {en}: trns: {:?}, health: {}", transform.pos, health.health);
    }
}