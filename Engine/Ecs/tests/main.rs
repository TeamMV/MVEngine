use mvcore::math::vec::Vec3;
use mvengine_ecs::{Entity, ECS};

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
    let health2 = player2.get_component::<Health>().unwrap();
    println!("Health2: {}", health2.health);
}