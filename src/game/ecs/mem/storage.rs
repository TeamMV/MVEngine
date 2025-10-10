use crate::game::ecs::entity::EntityId;
use crate::game::ecs::mem::conblob;
use crate::game::ecs::mem::conblob::ContinuousBlob;
use hashbrown::HashMap;
use itertools::Itertools;
use mvutils::hashers::U64IdentityHasher;
use mvutils::unsafe_utils::Unsafe;
use std::alloc::Layout;
use std::any::TypeId;
use std::fmt::Debug;

pub(crate) type ComponentIdx = u64;

#[derive(Hash, PartialEq, Eq)]
struct ComponentKey {
    type_id: TypeId,
    index: ComponentIdx,
}

pub struct ComponentStorage {
    components: HashMap<TypeId, ContinuousBlob>,
    entity_components:
        HashMap<EntityId, HashMap<TypeId, ComponentIdx, U64IdentityHasher>, U64IdentityHasher>,
    component_entities: HashMap<ComponentKey, EntityId>,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            entity_components: HashMap::with_hasher(U64IdentityHasher::default()),
            component_entities: HashMap::new(),
        }
    }

    pub fn get_component<T: Sized + 'static>(&self, entity: EntityId) -> Option<&T> {
        if let Some(map) = self.entity_components.get(&entity) {
            if let Some(idx) = map.get(&TypeId::of::<T>()) {
                if let Some(blob) = self.components.get(&TypeId::of::<T>()) {
                    return blob.get(*idx);
                }
            }
        }
        None
    }

    pub fn get_component_mut<T: Sized + 'static>(&mut self, entity: EntityId) -> Option<&mut T> {
        if let Some(map) = self.entity_components.get_mut(&entity) {
            if let Some(idx) = map.get_mut(&TypeId::of::<T>()) {
                if let Some(blob) = self.components.get_mut(&TypeId::of::<T>()) {
                    return blob.get_mut(*idx);
                }
            }
        }
        None
    }

    pub(crate) fn get_component_mut_bruh<T: Sized + 'static>(
        &self,
        entity: EntityId,
    ) -> Option<&mut T> {
        if let Some(map) = self.entity_components.get(&entity) {
            if let Some(idx) = map.get(&TypeId::of::<T>()) {
                if let Some(blob) = self.components.get(&TypeId::of::<T>()) {
                    return blob.get_mut_bruh(*idx);
                }
            }
        }
        None
    }

    pub fn set_component<T: Sized + 'static>(&mut self, entity: EntityId, component: T) {
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
                self.component_entities.insert(
                    ComponentKey {
                        type_id: TypeId::of::<T>(),
                        index: idx,
                    },
                    entity,
                );
                self.entity_components.get_mut(&entity).unwrap()
            };

            map.insert(TypeId::of::<T>(), idx as ComponentIdx);
        }
    }

    pub fn remove_component<T: Sized + 'static>(&mut self, entity: EntityId) {
        let type_id = TypeId::of::<T>();
        if let Some(blob) = self.components.get_mut(&type_id)
            && let Some(map) = self.entity_components.get(&entity)
            && let Some(idx) = map.get(&type_id)
        {
            blob.remove(*idx);
        }
    }

    pub fn remove_entity(&mut self, entity: EntityId) {
        if let Some(components) = self.entity_components.remove(&entity) {
            for (ty, idx) in components {
                if let Some(blob) = self.components.get_mut(&ty) {
                    blob.remove(idx);
                }
                let key = ComponentKey {
                    type_id: ty,
                    index: idx,
                };
                self.component_entities.remove(&key);
            }
        }
    }

    fn get_entity_from_component_instance<C: 'static>(
        &self,
        idx: ComponentIdx,
    ) -> Option<EntityId> {
        let t = TypeId::of::<C>();
        let key = ComponentKey {
            type_id: t,
            index: idx,
        };
        self.component_entities.get(&key).copied()
    }

    mvengine_proc_macro::generate_queries!(20);
}
