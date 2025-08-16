// fuck this compiler who gives a shit about naming conventions
#![allow(non_snake_case)]

use crate::game::ecs::entity::EntityId;
use crate::game::ecs::World;
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

impl<C1: Sized + 'static> System<(C1,)> {
    #[auto_enums::auto_enum(Iterator)]
    pub fn iter<'a>(&'a self, world: &'a World) -> impl Iterator<Item=(EntityId, (&C1 ))> + 'a {
        match world {
            World::SparseSet(ssw) => ssw.storage.query1::<C1>(),
            World::ArchetypeWorld(atw) => std::iter::empty(),
        }
    }
}

//mvengine_proc_macro::generate_system_impls!(20);