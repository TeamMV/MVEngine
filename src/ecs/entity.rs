use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use mvutils::once::CreateOnce;
use mvutils::unsafe_utils::{DangerousCell, Unsafe, UnsafeRef};
use mvutils::utils;
use crate::ecs::{EcsStorage, ECS};
use crate::ecs::mem::storage::ComponentStorage;

pub type EntityType = u64;

pub trait EntityBehavior {
    fn new(storage: EcsStorage) -> Self where Self: Sized;
    fn start(&mut self, entity: EntityType);
    fn update(&mut self, entity: EntityType);
}

#[derive(Clone)]
pub struct NoBehavior;

impl EntityBehavior for NoBehavior {
    fn new(storage: EcsStorage) -> Self
    where
        Self: Sized
    { Self {} }

    fn start(&mut self, entity: EntityType) {}

    fn update(&mut self, entity: EntityType) {}
}

#[derive(Clone)]
pub struct LocalComponent<C: Sized + 'static> {
    component: Option<&'static C>,
    storage: EcsStorage
}

impl<C: Sized + 'static> LocalComponent<C> {
    pub fn new(storage: EcsStorage) -> Self {
        Self { component: None, storage }
    }

    pub fn aquire(&mut self, entity: EntityType) {
        let c = self.storage.get().get_component::<C>(entity).expect("Entity does not have component X, but it was aquired from LocalComponent!");
        unsafe { self.component = Some(Unsafe::cast_static(c)) }
    }
}

impl<C> Deref for LocalComponent<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        unsafe { self.component.unwrap() }
    }
}

impl<C> DerefMut for LocalComponent<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { (*self.component.as_ref().unwrap() as *const C as *mut C).as_mut().unwrap() }
    }
}

pub struct Entity<B, C> {
    phantom: PhantomData<C>,
    pub(crate) ty: EntityType,
    storage: EcsStorage,
    pub(crate) behavior: Option<B>
}

impl<B, C> Entity<B, C> {
    pub fn get_component<T: Sized + 'static>(&self) -> Option<&T> {
        let mut st = self.storage.get_mut();
        st.get_component::<T>(self.ty)
    }

    pub fn get_component_mut<T: Sized + 'static>(&mut self) -> Option<&mut T> {
        let mut st = self.storage.get_mut();
        st.get_component_mut::<T>(self.ty)
    }
}

impl<B: EntityBehavior, C> Entity<B, C> {
    fn new_internal(storage: EcsStorage, behavior: Option<B>) -> Self {
        Self {
            phantom: PhantomData::default(),
            ty: utils::next_id("MVEngine::ecs::entity"),
            storage,
            behavior,
        }
    }

    pub fn start(&mut self) {
        if let Some(ref mut behavior) = self.behavior {
            behavior.start(self.ty);
        }
    }

    pub fn update(&mut self) {
        if let Some(ref mut behavior) = self.behavior {
            behavior.update(self.ty);
        }
    }

    pub fn get_behavior(&self) -> &B {
        self.behavior.as_ref().unwrap()
    }

    pub fn get_behavior_mut(&mut self) -> &mut B {
        &mut *self.behavior.as_mut().unwrap()
    }
}

macro_rules! impl_entity_tuples {
    ($first:ident) => {};

    ($first:ident $($rest:ident)*) => {
        impl_entity_tuples!($($rest)*);

        impl<B: EntityBehavior, $first: Sized + Default + 'static, $($rest: Sized + Default + 'static),*> Entity<B, ($first, $($rest),*)> {
            pub fn new(storage: EcsStorage) -> Self {
                let mut this = Self::new_internal(storage.clone(), Some(B::new(storage.clone())));

                #[allow(non_snake_case)]
                let ($first, $($rest),*) = ($first::default(), $($rest::default()),*);

                this.storage.get_mut().set_component(this.ty, $first);
                $( this.storage.get_mut().set_component(this.ty, $rest); )*

                this
            }
        }

        impl<B: EntityBehavior + Clone, $first: Sized + Default + 'static + Clone, $($rest: Sized + Default + 'static + Clone),*> Clone for Entity<B, ($first, $($rest),*)> {
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

impl<B: EntityBehavior, C: Sized + Default + 'static> Entity<B, (C,)> {
    pub fn new(storage: EcsStorage) -> Self {
        let mut this = Self::new_internal(storage.clone(), Some(B::new(storage.clone())));

        #[allow(non_snake_case)]
        let (c) = (C::default());

        this.storage.get_mut().set_component(this.ty, c);

        this
    }
}

impl<B: EntityBehavior + Clone, C: Sized + Clone + Default + 'static> Clone for Entity<B, (C,)> {
    fn clone(&self) -> Self {
        let component = self.get_component::<C>().unwrap();

        let mut new = Self::new(self.storage.clone());

        let mut new_component = new.get_component_mut::<C>().unwrap();
        *new_component = component.clone();

        new
    }
}