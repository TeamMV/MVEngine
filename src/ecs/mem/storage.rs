use crate::ecs::mem::conblob::ContinuousBlob;
use hashbrown::HashMap;
use mvutils::hashers::{U64IdentityHasher, UsizeIdentityHasher};
use std::alloc::Layout;
use std::any::TypeId;
use mvutils::unsafe_utils::Unsafe;
use mvengine_proc_macro::generate_get_components;
use crate::ecs::entity::EntityType;

pub(crate) type ComponentIdx = u64;

pub struct ComponentStorage {
    components: HashMap<TypeId, ContinuousBlob>,
    entity_components: HashMap<EntityType, HashMap<TypeId, ComponentIdx, U64IdentityHasher>, U64IdentityHasher>,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            entity_components: HashMap::with_hasher(U64IdentityHasher::default()),
        }
    }

    pub fn get_component<T: Sized + 'static>(&self, entity: EntityType) -> Option<&T> {
        if let Some(map) = self.entity_components.get(&entity) {
            if let Some(idx) = map.get(&TypeId::of::<T>()) {
                if let Some(blob) = self.components.get(&TypeId::of::<T>()) {
                    return blob.get(*idx as usize);
                }
            }
        }
        None
    }

    pub fn get_component_mut<T: Sized + 'static>(&mut self, entity: EntityType) -> Option<&mut T> {
        if let Some(map) = self.entity_components.get_mut(&entity) {
            if let Some(idx) = map.get_mut(&TypeId::of::<T>()) {
                if let Some(blob) = self.components.get_mut(&TypeId::of::<T>()) {
                    return blob.get_mut(*idx as usize);
                }
            }
        }
        None
    }

    pub fn set_component<T: Sized + 'static>(&mut self, entity: EntityType, component: T) {
        let blob = if let Some(blob) = self.components.get_mut(&TypeId::of::<T>()) {
            blob
        } else {
            let blob = ContinuousBlob::new(Layout::for_value(&component));
            self.components.insert(TypeId::of::<T>(), blob);
            self.components.get_mut(&TypeId::of::<T>()).unwrap()
        };

        if let Some(idx) = blob.push_next(component) {
            let map = if let Some(map) = self.entity_components.get_mut(&entity) {
                map
            } else {
                let map = HashMap::with_hasher(U64IdentityHasher::default());
                self.entity_components.insert(entity, map);
                self.entity_components.get_mut(&entity).unwrap()
            };

            map.insert(TypeId::of::<T>(), idx as ComponentIdx);
        }
    }

    generate_get_components!(15);

    /*pub fn get_components1<C1: Sized + 'static>(&self) -> Option<Vec<(EntityType, &C1)>> {
        let c1s = self.components.get(&TypeId::of::<C1>())?.get_all::<C1>();

        let mut out = vec![];

        for (en, map) in self.entity_components.iter() {
            if let Some(idx1) = map.get(&TypeId::of::<C1>()) {
                if let Some(c1) = c1s.get(*idx1) {
                    out.push((*en, *c1));
                }
            }
        }

        Some(out)
    }

    pub fn get_components1_mut<C1: Sized + 'static>(&mut self) -> Option<Vec<(EntityType, &mut C1)>> {
        let mut c1s = self.components.get_mut(&TypeId::of::<C1>())?.get_all_mut::<C1>();

        let mut out = vec![];

        for (en, map) in self.entity_components.iter_mut() {
            if let Some(idx1) = map.get_mut(&TypeId::of::<C1>()) {
                if let Some(c1) = c1s.get_mut(*idx1) {
                    unsafe { out.push((*en, Unsafe::cast_mut_static(*c1))); }
                }
            }
        }

        Some(out)
    }

    pub fn get_components2<C1: Sized + 'static, C2: Sized + 'static>(&self) -> Option<Vec<(EntityType, &C1, &C2)>> {
        let c1s = self.components.get(&TypeId::of::<C1>())?.get_all::<C1>();
        let c2s = self.components.get(&TypeId::of::<C2>())?.get_all::<C2>();

        let mut out = vec![];

        for (en, map) in self.entity_components.iter() {
            if let Some(idx1) = map.get(&TypeId::of::<C1>()) {
                if let Some(idx2) = map.get(&TypeId::of::<C2>()) {
                    if let Some(c1) = c1s.get(*idx1) {
                        if let Some(c2) = c2s.get(*idx2) {
                            out.push((*en, *c1, *c2));
                        }
                    }
                }
            }
        }

        Some(out)
    }

    pub fn get_components2_mut<C1: Sized + 'static, C2: Sized + 'static>(&mut self) -> Option<Vec<(EntityType, &mut C1, &mut C2)>> {
        let mut c1s = self.components.get(&TypeId::of::<C1>())?.get_all::<C1>();
        let mut c2s = self.components.get(&TypeId::of::<C2>())?.get_all::<C2>();

        let mut out = vec![];

        unsafe {
            for (en, map) in self.entity_components.iter_mut() {
                if let Some(idx1) = map.get(&TypeId::of::<C1>()) {
                    if let Some(idx2) = map.get(&TypeId::of::<C2>()) {
                        if let Some(c1) = c1s.get_mut(*idx1) {
                            if let Some(c2) = c2s.get_mut(*idx2) {
                                out.push((*en, (*c1 as *const C1 as *mut C1).as_mut().unwrap(), (*c2 as *const C2 as *mut C2).as_mut().unwrap()));
                            }
                        }
                    }
                }
            }
        }

        Some(out)
    }*/
}