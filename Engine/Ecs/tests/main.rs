use mvutils::unsafe_utils::UnsafeRef;
use mvcore::math::vec::Vec3;
use mvengine_ecs::{EcsStorage, ECS};
use mvengine_ecs::entity::{Entity, EntityBehavior, EntityType, LocalComponent, NoBehavior, StaticEntity};
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
    }

    fn update(&mut self, entity: EntityType) {
        self.health.health += 1f32;
    }
}

fn main() {
    let ecs = ECS::new();

    let mut player1 = Entity::<PlayerBehavior, (Health, Transform)>::new(ecs.storage());
    let mut player2 = player1.clone();
    let mut block1 = StaticEntity::<(Transform,)>::new(ecs.storage());

    player1.start();
    player1.update();

    let my_system = System::<(Transform, Health)>::new(ecs.storage());
    for (en, transform, health) in my_system.iter() {
        println!("entity: {en}: trns: {:?}, health: {}", transform.pos, health.health);
    }
}