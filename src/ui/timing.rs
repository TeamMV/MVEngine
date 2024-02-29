use crate::ui::background::{BackgroundEffect, BackgroundEffectInfo};
use crate::ui::styles::Point;
use hashbrown::HashMap;
use mvsync::MVSync;
use mvutils::hashers::U64IdentityHasher;
use mvutils::lazy;
use mvutils::utils::Time;
use std::marker::PhantomData;
use std::sync::Arc;

pub struct TimingManager {
    tasks: HashMap<u64, Box<dyn TimingTask>, U64IdentityHasher>,
}

lazy! {
    pub static mut TIMING_MANAGER: TimingManager = TimingManager::new();
}

impl TimingManager {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::with_hasher(U64IdentityHasher::default()),
        }
    }

    pub fn request<T>(&mut self, task: T) -> u64
    where
        T: TimingTask + 'static,
    {
        let id = mvutils::utils::next_id("MVCore::ui::timingTask");
        self.tasks.insert(id, Box::new(task));
        id
    }

    pub fn cancel(&mut self, id: u64) {
        self.tasks.remove(&id);
    }

    pub fn do_frame(&mut self, dt: f32, frame: u64) {
        let mut to_remove = vec![];
        for task in self.tasks.iter_mut() {
            task.1.iteration(dt, frame);
            if task.1.is_done() {
                to_remove.push(task.0.clone());
            }
        }
        for id in to_remove {
            self.tasks.remove(&id);
        }
    }
}

pub trait TimingTask {
    fn is_done(&self) -> bool;
    fn iteration(&mut self, dt: f32, frame: u64);
}

pub struct IterationTask {
    function: Box<dyn Fn(&EffectState, u32)>,
    state: EffectState,
    current_iter: u32,
    limit: u32,
}

impl IterationTask {
    pub fn new<F>(limit: u32, function: F, state: EffectState) -> Self
    where
        F: Fn(&EffectState, u32) + 'static,
    {
        Self {
            current_iter: 0,
            limit,
            state,
            function: Box::new(function),
        }
    }
}

impl TimingTask for IterationTask {
    fn is_done(&self) -> bool {
        self.current_iter >= self.limit
    }

    fn iteration(&mut self, dt: f32, frame: u64) {
        (self.function)(&self.state, self.current_iter);
        self.current_iter += 1;
    }
}

pub struct DurationTask {
    function: Box<dyn Fn(&EffectState, u32)>,
    state: EffectState,
    init_time: u128,
    duration: u32,
}

impl DurationTask {
    pub fn new<F>(duration_ms: u32, function: F, state: EffectState) -> Self
    where
        F: Fn(&EffectState, u32) + 'static,
    {
        Self {
            function: Box::new(function),
            state,
            init_time: u128::time_millis(),
            duration: duration_ms,
        }
    }
}

impl TimingTask for DurationTask {
    fn is_done(&self) -> bool {
        u128::time_millis() > self.init_time + self.duration as u128
    }

    fn iteration(&mut self, dt: f32, frame: u64) {
        (self.function)(&self.state, (u128::time_millis() - self.init_time) as u32);
    }
}

pub struct DelayTask {
    function: Box<dyn Fn(&EffectState, u32, u32)>,
    state: EffectState,
    init_time: u128,
    duration: u32,
    delay: u32,
    current_iter: u32,
}

impl DelayTask {
    pub fn new<F>(duration_ms: u32, delay_ms: u32, function: F, state: EffectState) -> Self
    where
        F: Fn(&EffectState, u32, u32) + 'static,
    {
        Self {
            function: Box::new(function),
            state,
            init_time: u128::time_millis(),
            duration: duration_ms,
            delay: delay_ms,
            current_iter: 0,
        }
    }
}

impl TimingTask for DelayTask {
    fn is_done(&self) -> bool {
        u128::time_millis() > self.init_time + self.duration as u128
    }

    fn iteration(&mut self, dt: f32, frame: u64) {
        let ms = u128::time_millis();
        if ms - self.init_time > (self.delay * (self.current_iter + 1)) as u128 {
            (self.function)(&self.state, (ms - self.init_time) as u32, self.current_iter);
            self.current_iter += 1;
        }
    }
}

pub(crate) struct EffectState {
    pub(crate) background: Option<BackgroundEffectInfo>,
}

impl EffectState {
    pub(crate) fn background(bg: BackgroundEffectInfo) -> Self {
        Self {
            background: Some(bg),
        }
    }
}
