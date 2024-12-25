use mvcore::math::vec::Vec3;
use mvengine_ecs::entity::{Entity, EntityBehavior, EntityType, LocalComponent};
use mvengine_ecs::{EcsStorage, ECS};
use mvengine_ecs::system::System;

#[derive(Default, Clone)]
struct Health {
    pub health: f32
}

#[derive(Default, Clone)]
struct Transform {
    pub pos: Vec3
}

#[derive(Clone)]
struct PlayerBehavior {
    health: LocalComponent<Health>,
    transform: LocalComponent<Transform>,
}

impl EntityBehavior for PlayerBehavior {
    fn new(storage: EcsStorage) -> Self {
        Self {
            health: LocalComponent::new(storage.clone()),
            transform: LocalComponent::new(storage),
        }
    }

    fn start(&mut self, entity: EntityType) {
        self.health.aquire(entity);
        self.transform.aquire(entity);
        println!("Starting...");
    }

    fn update(&mut self, entity: EntityType) {
        self.health.health += 1f32;
    }
}

fn main() {
    let mut ecs = ECS::new();
    let world = ecs.world_mut();
    world.create_entity(|s| Entity::<PlayerBehavior, (Health, Transform)>::new(s));

    for _ in 0..5 {
        world.update();
    }

    let system = System::<(Health,)>::new(ecs.storage());
    for (en, health) in system.iter() {
        println!("Entity: {en}: health: {}", health.health);
    }
}