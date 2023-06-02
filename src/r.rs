//use std::collections::HashMap;
//use std::sync::RwLock;
//use crate::render::common::TextureRegion;
//
//pub(crate) struct Ref<T> {
//    t: Lazy<RwLock<HashMap<String, T>>>
//}
//
//impl<T> Ref<T> {
//    pub fn new() -> Self {
//        Self {
//            t: Lazy::new(|| RwLock::new(HashMap::new())),
//        }
//    }
//
//    pub fn add(&self, res_id: &str, t: T) {
//        let mut lock = self.t.write().unwrap();
//        lock.insert(res_id.to_string(), t);
//    }
//
//    pub fn get(&self, res_id: &str) -> Option<&T> {
//        let mut lock = self.t.read().unwrap();
//        lock.get(res_id)
//    }
//
//    pub fn get_mut(&self, res_id: &str) -> Option<&mut T> {
//        let mut lock = self.t.read().unwrap();
//        lock.get_mut(res_id)
//    }
//}
//
//pub static mut TEXTURES: Ref<TextureRegion> = Ref::new();