use crate::entity::Entity;
use crate::system::{System, SystemBase, ThreadedSystem};

pub mod entity;
pub mod component;
pub mod system;

pub struct ECS {
    entities: Vec<Box<(dyn Entity + Send + Sync)>>,
    joined_systems: Vec<Box<(dyn System + Send + Sync)>>,
    threaded_systems: Vec<Box<(dyn System + Send + Sync)>>,
    index_mapper: Vec<(bool, usize)>
}

impl ECS {
    pub fn new() -> Self {
        Self {
            entities: vec![],
            joined_systems: vec![],
            threaded_systems: vec![],
            index_mapper: vec![],
        }
    }

    pub fn insert_system<S: System + Send + Sync + 'static>(&mut self, mut sys: S) {
        sys.init();

        let mut base = sys.get_base_mut();
        base.init(self, self.index_mapper.len());

        if let SystemBase::Joined(ref j) = base {
            self.joined_systems.push(Box::new(sys));
            self.index_mapper.push((true, self.joined_systems.len() - 1));
        } else {
            self.threaded_systems.push(Box::new(sys));
            self.index_mapper.push((false, self.threaded_systems.len() - 1));
        }
    }

    pub fn insert_entity<E: Entity + Send + Sync + 'static>(&mut self, mut entity: E) {
        entity.init();
        self.entities.push(Box::new(entity));
    }

    pub fn system_at(&self, idx: usize) -> &Box<(dyn System + Send + Sync)> {
        let full_idx = self.index_mapper[idx];
        if full_idx.0 {
            return &self.joined_systems[full_idx.1];
        }
        &self.threaded_systems[full_idx.1]
    }

    pub fn system_at_mut(&mut self, idx: usize) -> &mut Box<(dyn System + Send + Sync)> {
        let full_idx = self.index_mapper[idx];
        if full_idx.0 {
            return &mut self.joined_systems[full_idx.1];
        }
        &mut self.threaded_systems[full_idx.1]
    }
}

impl Behavior for ECS {
    fn init(&mut self) {
        self.entities.iter_mut().for_each(|e| e.init());
        self.joined_systems.iter_mut().for_each(|e| e.init());
    }

    fn update(&mut self) {
        for en in self.entities.iter_mut() {
            en.update();

            for sys in self.joined_systems.iter_mut() {
                if sys.entity_valid(en.get_base()) {
                    sys.check_entity(en);
                }
            }
        }
    }
}

pub trait Behavior {
    fn init(&mut self);

    fn update(&mut self);
}