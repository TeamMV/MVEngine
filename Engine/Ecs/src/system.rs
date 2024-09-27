use crate::{Behavior, ECS};
use mvutils::clock::Clock;
use mvutils::unsafe_utils::{DangerousCell, Unsafe, UnsafeRef};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use crate::entity::{Entity, EntityBase};

pub trait System: SystemBehavior {
    fn get_base(&self) -> &SystemBase;
    fn get_base_mut(&mut self) -> &mut SystemBase;

    fn entity_valid(&self, en: &EntityBase) -> bool;
}

pub enum SystemBase {
    Joined(JoinedSystem),
    Threaded(ThreadedSystem)
}

impl SystemBase {
    pub fn init(&mut self, ecs: &ECS, sys_idx: usize) {
        match self {
            SystemBase::Joined(s) => s.init(),
            SystemBase::Threaded(s) => s.init(ecs, sys_idx)
        }
    }
}

pub trait SystemBehavior {
    fn init(&mut self);

    fn check_entity(&self, en: &Box<(dyn Entity + Send + Sync)>);
}

pub struct JoinedSystem {
    clock: Clock
}

impl JoinedSystem {
    pub fn new(tps: u16) -> Self {
        Self {
            clock: Clock::new(tps),
        }
    }

    fn init(&mut self) {
        self.clock.start()
    }
}

pub struct ThreadedSystem {
    clock: Clock,
    handle: Option<JoinHandle<()>>,
    break_thread: AtomicBool,
    ecs: Option<&'static ECS>,
    sys_idx: usize,
}

impl ThreadedSystem {
    pub fn new(tps: u16) -> Self {
        let mut clock = Clock::new(tps);

        let clock_ref = unsafe { Unsafe::cast_mut_static(&mut clock) };

        let mut this = Self {
            clock,
            handle: None,
            break_thread: AtomicBool::new(false),
            ecs: None,
            sys_idx: 0,
        };

        let this_ref = unsafe { Unsafe::cast_mut_static(&mut this) };

        let handle = thread::spawn(|| {
            loop {
                if this_ref.break_thread.load(Ordering::Acquire) {
                    break;
                }

                clock_ref.wait_tick(0);
                this_ref.update_async();
            }
        });

        this.handle = Some(handle);

        this
    }

    pub fn init(&mut self, ecs: &ECS, sys_idx: usize) {
        self.clock.start();

        self.sys_idx = sys_idx;
        unsafe {
            self.ecs = Some(Unsafe::cast_static(ecs));
        }
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().expect("Oh no! ThreadedSystem panicked!");
        }
    }

    fn update_async(&mut self) {
        if let Some(mut e) = self.ecs {
            let sys = e.system_at(self.sys_idx);

            for entity in e.entities.iter() {
                if sys.entity_valid(entity.get_base()) {
                    sys.check_entity(entity);
                }
            }
        }
    }
}