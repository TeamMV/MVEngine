use std::marker::PhantomData;
use std::sync::Arc;
use mvutils::unsafe_utils::DangerousCell;
use mvutils::utils;
use crate::mem::storage::{ComponentStorage, EntityType};

pub mod component;
mod mem;

pub struct ECS {
    pub(crate) storage: Arc<DangerousCell<ComponentStorage>>,
}

impl ECS {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(DangerousCell::new(ComponentStorage::new()))
        }
    }

    pub fn storage(&self) -> Arc<DangerousCell<ComponentStorage>> {
        self.storage.clone()
    }
}

pub struct Entity<C> {
    phantom: PhantomData<C>,
    ty: EntityType,
    storage: Arc<DangerousCell<ComponentStorage>>,
}

impl<C> Entity<C> {
    fn new_internal(storage: Arc<DangerousCell<ComponentStorage>>) -> Self {
        Self {
            phantom: PhantomData::default(),
            ty: utils::next_id("MVEngine::ecs::entity"),
            storage,
        }
    }

    pub fn get_component<T: Sized + 'static>(&self) -> Option<&T> {
        let mut st = self.storage.get_mut();
        st.get_component::<T>(self.ty)
    }

    pub fn get_component_mut<T: Sized + 'static>(&mut self) -> Option<&mut T> {
        let mut st = self.storage.get_mut();
        st.get_component_mut::<T>(self.ty)
    }
}

macro_rules! impl_entity_tuples {
    () => {};
    ($first:ident $($rest:ident)*) => {
        impl_entity_tuples!($($rest)*);

        impl<$first: Sized + Default + 'static, $($rest: Sized + Default + 'static),*> Entity<($first, $($rest),*)> {
            pub fn new(storage: Arc<DangerousCell<ComponentStorage>>) -> Self {
                let mut this = Self::new_internal(storage);

                #[allow(non_snake_case)]
                let ($first, $($rest),*) = ($first::default(), $($rest::default()),*);

                this.storage.get_mut().set_component(this.ty, $first);
                $( this.storage.get_mut().set_component(this.ty, $rest); )*

                this
            }
        }

        impl<$first: Sized + Default + 'static + Clone, $($rest: Sized + Default + 'static + Clone),*> Clone for Entity<($first, $($rest),*)> {
            fn clone(&self) -> Self {
                let component = self.get_component::<$first>().unwrap();

                let mut new = Self::new(self.storage.clone());

                let mut new_component = new.get_component_mut::<$first>().unwrap();
                component.clone_into(&mut new_component);

                $(
                    let mut component = self.get_component::<$rest>().unwrap();
                    let mut new_component = new.get_component_mut::<$rest>().unwrap();
                    component.clone_into(&mut new_component);
                )*

                new
            }
        }
    };
}

impl_entity_tuples!(C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12 C13 C14 C15);