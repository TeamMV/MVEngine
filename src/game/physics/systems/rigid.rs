use crate::game::ecs::system::System;
use crate::game::ecs::World;
use crate::game::physics::components::{RigidDynamic, Transform};
use crate::math::vec::Vec2;
use crate::ui::geometry::geom;

pub struct RigidSystem {
    s: System<(Transform, RigidDynamic)>
}

impl RigidSystem {
    pub fn new() -> Self {
        Self {
            s: System::new(),
        }
    }

    pub fn iterate(&mut self, world: &mut World, dt: f64) {
        for (entity, (trns, dy)) in self.s.iter_mut(world) {
            if !geom::is_vec_zero(dy.velocity) {
                Self::apply_vel(&mut trns.position, dy.velocity, dt);
            }
            if !geom::is_vec_zero(dy.gravity) {
                Self::apply_vel(&mut trns.position, dy.gravity, dt);
            }
            if dy.circular_velocity != 0.0 {
                trns.rotation += dy.circular_velocity * dt;
            }
        }
    }

    fn apply_vel(target: &mut Vec2, vel: Vec2, dt: f64) {
        target.x += vel.x * dt as f32;
        target.y += vel.y * dt as f32;
    }
}