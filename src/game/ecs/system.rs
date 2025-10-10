// fuck this compiler who gives a shit about naming conventions
#![allow(non_snake_case)]

use crate::game::ecs::World;
use crate::game::ecs::entity::EntityId;
use std::fmt::Debug;
use std::marker::PhantomData;

pub struct System<C> {
    phantom: PhantomData<C>,
}

impl<C> System<C> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData::default(),
        }
    }
}

mvengine_proc_macro::generate_system_impls!(20);
