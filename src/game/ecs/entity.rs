use mvutils::utils;
use std::marker::PhantomData;
use crate::game::ecs::World;
use crate::game::ecs::world::EcsWorld;

pub type EntityId = u64;

pub struct Entity<C> {
    phantom: PhantomData<C>,
    pub(crate) ty: EntityId,
}

impl<C> Entity<C> {
    fn new_internal() -> Self {
        Self {
            phantom: PhantomData::default(),
            ty: utils::next_id("MVEngine::ecs::entity"),
        }
    }
}

macro_rules! impl_entity_tuples {
    ($first:ident,) => {};

    ($first:ident, $($rest:ident, )*) => {
        impl_entity_tuples!($($rest,)*);

        impl<$first: Sized + Default + 'static, $($rest: Sized + Default + 'static),*> Entity<($first, $($rest),*)> {
            pub fn create(world: &mut World) -> EntityId {
                let this = Self::new_internal();

                #[allow(non_snake_case)]
                let ($first, $($rest),*) = ($first::default(), $($rest::default()),*);

                world.set_component(this.ty, $first);
                $( world.set_component(this.ty, $rest); )*

                this.ty
            }
        }
    };
}

impl_entity_tuples!(
    C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17, C18, C19, C20,
);

impl<C: Sized + Default + 'static> Entity<(C,)> {
    pub fn create(world: &mut World) -> Self {
        let this = Self::new_internal();

        #[allow(non_snake_case)]
        let c = C::default();

        world.create_entity(this.ty);
        world.set_component(this.ty, c);

        this
    }
}