use crate::game::ecs::system::System;
use crate::game::ecs::World;
use crate::game::physics::components::{AABBCollider, Transform};
use crate::math::vec::Vec2;

pub struct AabbCollisionSystem {
    system: System<(Transform, AABBCollider)>,
}

impl AabbCollisionSystem {
    pub fn new() -> Self {
        Self {
            system: System::new(),
        }
    }

    pub fn iterate(&mut self, world: &mut World, dt: f64) {
        let mut entities: Vec<_> = self.system.iter_mut(world).collect();

        for (_, (_, collider)) in entities.iter_mut() {
            collider.collides = false;
            collider.overlap = Vec2::default();
            collider.collision_point = Vec2::default();
        }

        let mut collisions = vec![];
        let len = entities.len();

        // This is like super inefficient and should be replaced by some smart BVH or smth
        for i in 0..len {
            let (e1, (t1, c1)) = &entities[i];
            for j in (i + 1)..len {
                let (e2, (t2, c2)) = &entities[j];

                if let Some((overlap, mid)) = Self::check_collision(t1, c1, t2, c2) {
                    collisions.push((i, j, overlap, mid));
                }
            }
        }

        for (i, j, overlap, mid) in collisions {
            let (_, (_, c1)) = &mut entities[i];
            c1.collides = true;
            c1.overlap = overlap;
            c1.collision_point = mid;

            let (_, (_, c2)) = &mut entities[j];
            c2.collides = true;
            c2.overlap = overlap;
            c2.collision_point = mid;
        }
    }

    fn check_collision(
        t1: &Transform,
        c1: &AABBCollider,
        t2: &Transform,
        c2: &AABBCollider,
    ) -> Option<(Vec2, Vec2)> {
        let min1 = t1.position - c1.extent * 0.5;
        let max1 = t1.position + c1.extent * 0.5;
        let min2 = t2.position - c2.extent * 0.5;
        let max2 = t2.position + c2.extent * 0.5;

        if min1.x < max2.x && max1.x > min2.x &&
            min1.y < max2.y && max1.y > min2.y
        {
            let overlap_x = (max1.x.min(max2.x) - min1.x.max(min2.x)).max(0.0);
            let overlap_y = (max1.y.min(max2.y) - min1.y.max(min2.y)).max(0.0);
            let overlap = Vec2::new(overlap_x, overlap_y);

            let mid_x = (min1.x.max(min2.x) + max1.x.min(max2.x)) * 0.5;
            let mid_y = (min1.y.max(min2.y) + max1.y.min(max2.y)) * 0.5;
            let mid = Vec2::new(mid_x, mid_y);

            Some((overlap, mid))
        } else {
            None
        }
    }
}