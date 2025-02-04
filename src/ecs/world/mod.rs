use std::alloc::Layout;
use std::any::TypeId;
use std::process::id;
use hashbrown::HashMap;
use mvutils::hashers::U64IdentityHasher;
use crate::ecs::EcsStorage;
use crate::ecs::entity::{Entity, EntityBehavior, EntityType, NoBehavior};
use crate::ecs::mem::conblob::ContinuousBlob;

pub struct World {
    storage: EcsStorage,
    entities: Vec<EntityType>,
    behaviors: HashMap<TypeId, (ContinuousBlob, std::ptr::DynMetadata<dyn EntityBehavior>)>,
    behavior_indices: HashMap<EntityType, usize, U64IdentityHasher>,
    behavior_indices_rev: HashMap<u64, EntityType, U64IdentityHasher>
}

impl World {
    pub(crate) fn new(storage: EcsStorage) -> Self {
        Self {
            storage,
            entities: Vec::new(),
            behaviors: HashMap::new(),
            behavior_indices: HashMap::with_hasher(U64IdentityHasher::default()),
            behavior_indices_rev: HashMap::with_hasher(U64IdentityHasher::default()),
        }
    }

    pub fn update(&mut self) {
        for (behavior_blob, meta) in self.behaviors.values_mut() {
            let mut behaviors = behavior_blob.get_all_traits_mut::<dyn EntityBehavior>(meta);
            for (idx, behavior) in behaviors.iter_mut().enumerate() {
                if let Some(en_ty) = self.behavior_indices_rev.get(&(idx as u64)) {
                    behavior.update(*en_ty);
                }
            }
        }
    }

    pub fn create_entity<B: EntityBehavior + 'static, C>(&mut self, entity: fn(EcsStorage) -> Entity<B, C>) -> Option<()> {
        let mut entity = entity(self.storage.clone());
        let type_id = TypeId::of::<B>();
        let entity_ty = entity.ty;
        let b = entity.behavior;
        if b.is_some() {
            let mut b = b.unwrap();
            let idx = if let Some((cb, _)) = self.behaviors.get_mut(&type_id) {
                cb.push_next(b)?
            } else {
                let mut cb = ContinuousBlob::new(Layout::for_value(&b));
                let meta = std::ptr::metadata::<dyn EntityBehavior>(&b);
                let idx = cb.push_next(b)?;
                self.behaviors.insert(type_id, (cb, meta));
                idx
            };
            let br = self.behaviors.get_mut(&type_id).unwrap().0.get_mut::<B>(idx).unwrap();
            self.behavior_indices.insert(entity_ty, idx);
            self.behavior_indices_rev.insert(idx as u64, entity_ty);
            br.start(entity_ty);
        }

        Some(())
    }
}