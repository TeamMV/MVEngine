use crate::game::ecs::World;
use crate::game::physics::systems::aabb::AabbCollisionSystem;
use crate::game::physics::systems::rigid::RigidSystem;

pub mod aabb;
pub mod rigid;

pub struct PhysicsSystem {
    aabb_system: AabbCollisionSystem,
    rigid_system: RigidSystem
}

impl PhysicsSystem {
    pub fn new() -> Self {
        Self {
            aabb_system: AabbCollisionSystem::new(),
            rigid_system: RigidSystem::new(),
        }
    }

    pub fn iterate(&mut self, world: &mut World, dt: f64) {
        self.rigid_system.iterate(world, dt);
        self.aabb_system.iterate(world, dt);
    }
}