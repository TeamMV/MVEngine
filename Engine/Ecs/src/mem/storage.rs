use crate::mem::conblob::ContinuousBlob;
use hashbrown::HashMap;
use std::alloc::Layout;
use std::any::TypeId;

pub(crate) type EntityType = u64;
pub(crate) type ComponentIdx = usize;

pub struct ComponentStorage {
    components: HashMap<TypeId, ContinuousBlob>,
    entity_components: HashMap<EntityType, ComponentIdx>,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            entity_components: HashMap::new(),
        }
    }

    pub fn get_component<T: Sized + 'static>(&self, entity: EntityType) -> Option<&T> {
        if let Some(idx) = self.entity_components.get(&entity) {
            if let Some(blob) = self.components.get(&TypeId::of::<T>()) {
                return blob.get(*idx);
            }
        }
        None
    }

    pub fn get_component_mut<T: Sized + 'static>(&mut self, entity: EntityType) -> Option<&mut T> {
        if let Some(idx) = self.entity_components.get_mut(&entity) {
            if let Some(blob) = self.components.get_mut(&TypeId::of::<T>()) {
                return blob.get_mut(*idx);
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
            self.entity_components.insert(entity, idx);
        }
    }

    pub fn get_all_components<T: Sized + 'static>(&self) -> Option<Vec<&T>> {
        if let Some(blob) = self.components.get(&TypeId::of::<T>()) {
            let f = blob.get_all::<T>();
            return Some(f);
        }
        None
    }

    pub fn get_all_components_mut<T: Sized + 'static>(&mut self) -> Option<Vec<&mut T>> {
        if let Some(blob) = self.components.get_mut(&TypeId::of::<T>()) {
            let f = blob.get_all_mut::<T>();
            return Some(f);
        }
        None
    }
}