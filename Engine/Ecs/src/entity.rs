use std::any::{Any, TypeId};
use std::ffi::c_void;
use hashbrown::HashMap;
use crate::Behavior;
use crate::component::Component;

pub trait Entity: Behavior {
    fn get_base(&self) -> &EntityBase;

    fn get_base_mut(&mut self) -> &mut EntityBase;
}

pub struct EntityBase {
    components: HashMap<TypeId, *mut c_void>
}

unsafe impl Send for EntityBase {}
unsafe impl Sync for EntityBase {}

impl EntityBase {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn add_component<T: Component + 'static>(&mut self, component: &T) {
        let ptr = unsafe { component as *const (dyn Component + 'static) as *mut c_void };
        let id = TypeId::of::<T>();
        self.components.insert(id, ptr);
    }

    pub fn get_component<T: Component + 'static>(&self) -> Option<&T> {
        let id = TypeId::of::<T>();
        unsafe {
            self.components.get(&id).map(|ptr| &*(*ptr as *const T))
        }
    }

    pub fn get_component_mut<T: Component + 'static>(&mut self) -> Option<&mut T> {
        let id = TypeId::of::<T>();
        unsafe {
            self.components.get_mut(&id).map(|ptr| (*ptr as *mut T).as_mut().unwrap())
        }
    }

    pub fn has_component<T: Component + 'static>(&self) -> bool {
        let id = TypeId::of::<T>();
        self.components.contains_key(&id)
    }
}