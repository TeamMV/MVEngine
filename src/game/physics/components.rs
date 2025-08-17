use crate::math::vec::Vec2;

pub type Radians = f64;
pub type Kg = f64;

#[derive(Clone, Default, Debug, PartialOrd, PartialEq)]
pub struct Transform {
    /// Position relative to center.
    pub position: Vec2,
    /// Rotation around center in radiants.
    pub rotation: Radians,
    pub center: Vec2,
    /// The scale relative to center.
    pub scale: Vec2,
}

#[derive(Clone, Default, Debug, PartialOrd, PartialEq)]
pub struct RigidDynamic {
    pub velocity: Vec2,
    pub circular_velocity: Radians,
    pub gravity: Vec2,
    pub mass: Kg
}

#[derive(Clone, Default, Debug, PartialOrd, PartialEq)]
pub struct AABBCollider {
    /// The extent of this object as a rectangle with Transform::center as the middle.
    pub extent: Vec2,
    /// If this collider is colliding with another aabb collider right now
    pub collides: bool,
    /// The middle of the overlapping rect
    pub collision_point: Vec2,
    /// How much the foreign collider penetrates this collider
    pub overlap: Vec2
}